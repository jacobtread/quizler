use crate::{
    games::Games,
    msg::ServerEvent,
    session::{EventTarget, SessionId},
    types::{
        Answer, AnswerData, AnswerValue, GameToken, HostAction, ImStr, Image, ImageRef,
        NameFiltering, Question, QuestionData, RemoveReason, Score, ScoreCollection, ServerError,
    },
};
use log::debug;
use rustrict::CensorStr;
use serde::Serialize;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{sync::RwLock, task::AbortHandle, time::sleep};
use uuid::Uuid;

/// Reference to a game behind an Arc and a RwLock
pub type GameRef = Arc<RwLock<Game>>;

/// Represents an active quiz
pub struct Game {
    /// The token this game is stored behind
    token: GameToken,
    /// The host session
    host: HostSession,
    /// Map of session IDs mapped to the session address
    players: Vec<PlayerSession>,
    /// Configuration for the game
    config: Arc<GameConfig>,
    /// The state of the game
    state: GameState,
    /// The index of the current question
    question_index: usize,
    /// Spawn handle for delayed tasks
    task_handle: Option<AbortHandle>,
    /// Start time updated for each question
    start_time: Instant,
}

/// Different game states
#[derive(Serialize, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    /// The game is in the lobby
    Lobby,
    /// The game is starting
    Starting,
    /// The game is waiting for ready from all the players
    AwaitingReady,
    /// The question is about to start
    PreQuestion,
    /// The game has started and is waiting for answers
    AwaitingAnswers,
    /// The questions have been marked
    Marked,
    /// The game has finished
    Finished,
    /// The game has completely stopped
    Stopped,
}

/// Configuration data for a game
#[derive(Serialize)]
pub struct GameConfig {
    /// The name of the game
    pub name: ImStr,
    /// Text displayed under the game name
    pub text: ImStr,
    /// Maximum number of players allowed in this game
    pub max_players: usize,
    /// Filtering on names
    #[serde(skip)]
    pub filtering: NameFiltering,
    /// The game questions
    #[serde(skip)]
    pub questions: Box<[Arc<Question>]>,
    /// Map of uploaded image UUIDs to their respective
    /// image data
    #[serde(skip)]
    pub images: HashMap<ImageRef, Image>,
}

impl GameConfig {
    const MAX_TITLE_LENGTH: usize = 70;
    const MAX_DESCRIPTION_LENGTH: usize = 150;
    const MAX_QUESTIONS: usize = 50;

    /// Validates that the game configuration is valid
    /// and can be used for a game
    pub fn validate(&self) -> bool {
        if self.name.len() > Self::MAX_TITLE_LENGTH {
            return false;
        }

        if self.text.len() > Self::MAX_DESCRIPTION_LENGTH {
            return false;
        }

        let questions_length = self.questions.len();
        if questions_length == 0 || questions_length > Self::MAX_QUESTIONS {
            return false;
        }

        self.questions.iter().all(|value| value.validate())
    }
}

impl Game {
    /// Creates a new game instance
    ///
    /// # Arguments
    /// * token - The token for this game
    /// * host_id - The session ID of the host player
    /// * host_addr - The event target of the host player
    /// * config - The config for the game
    pub fn new(
        token: GameToken,
        host_id: SessionId,
        host_addr: EventTarget,
        config: Arc<GameConfig>,
    ) -> Self {
        Self {
            token,
            host: HostSession::new(host_id, host_addr),
            players: Default::default(),
            config,
            state: GameState::Lobby,
            question_index: 0,
            task_handle: None,
            start_time: Instant::now(),
        }
    }

    /// Creates a new delayed task to move to the next state once the provided
    /// duration has passed. This updates the timer state for clients as well
    ///
    /// # Arguments
    /// * duration - The duration to wait before moving states
    fn timed_next_state(&mut self, duration: Duration) {
        let token = self.token;
        let handle = tokio::spawn(async move {
            sleep(duration).await;
            let game = Games::get_game(&token).await;
            if let Some(game) = game {
                let lock = &mut *game.write().await;
                lock.task_handle = None;
                lock.next_state();
            }
        });

        self.task_handle = Some(handle.abort_handle());

        // Send timer message with the duration time
        self.send_all(ServerEvent::Timer {
            value: duration.as_millis() as u32,
        });
    }

    /// Handle resuming the game session for the player by `session_id` sends the
    /// relevant information to get the player up to speed
    pub fn resume_player(&self, session_id: SessionId) {
        let host = self.host.id == session_id;

        let mut target_addr: Option<&EventTarget> = None;
        let mut target_player: Option<&PlayerSession> = None;

        if host {
            target_addr = Some(&self.host.addr);
        } else {
            for player in &self.players {
                if player.id == session_id {
                    target_addr = Some(&player.addr);
                    target_player = Some(player);
                    break;
                }
            }
        }

        let target_addr = match target_addr {
            Some(player) => player,
            None => return,
        };

        // Notify the player they have resumed a game
        target_addr.send(Arc::new(ServerEvent::ResumedGame {
            id: session_id,
            host,
            token: self.token,
            config: self.config.clone(),
            name: target_player.as_ref().map(|player| player.name.clone()),
        }));

        // Notify the player of all the other players in the game
        for player in &self.players {
            target_addr.send(Arc::new(ServerEvent::PlayerData {
                id: player.id,
                name: player.name.clone(),
            }));
        }

        // Update everyone's scores
        // (Must come before game state so that the frontend can compute finished scores)
        let scores: Vec<(SessionId, u32)> = self
            .players
            .iter()
            .map(|player| (player.id, player.score))
            .collect();
        let scores = ScoreCollection(scores);
        target_addr.send(Arc::new(ServerEvent::Scores { scores }));

        // Send the player the current game state
        target_addr.send(Arc::new(ServerEvent::GameState { state: self.state }));

        match self.state {
            // Send the current question to the player for states that depend on one
            GameState::Starting
            | GameState::AwaitingReady
            | GameState::PreQuestion
            | GameState::Marked => {
                let question = self.config.questions[self.question_index].clone();
                target_addr.send(Arc::new(ServerEvent::Question { question }));

                // Provide the answer score if the question has been scored
                // (Only for players and not the host)
                if let Some(target_player) = target_player {
                    let answer = target_player.answers.get_answer_ref(self.question_index);
                    if let Some(score) = answer.score {
                        target_addr.send(Arc::new(ServerEvent::Score { score }));
                    }
                }
            }

            _ => {}
        }
    }

    /// Moves the game to the next state based on its current state
    fn next_state(&mut self) {
        // Cancel a delayed task if one is running
        if let Some(task_handle) = self.task_handle.take() {
            task_handle.abort();
        }

        match self.state {
            // Next state after lobby is starting
            GameState::Lobby => {
                const START_DURATION: Duration = Duration::from_secs(5);

                self.set_state(GameState::Starting);
                self.timed_next_state(START_DURATION);
            }

            // Next state after starting is question
            GameState::Starting => {
                self.question();
            }

            // Next state after awaiting ready is pre question
            GameState::AwaitingReady => {
                const START_DURATION: Duration = Duration::from_secs(5);
                self.set_state(GameState::PreQuestion);
                self.timed_next_state(START_DURATION);
            }

            // Next state after pre-question is awaiting answers
            GameState::PreQuestion => {
                // Await answers for the question
                self.set_state(GameState::AwaitingAnswers);

                // Assign the question start time
                self.start_time = Instant::now();

                let question = &self.config.questions[self.question_index];
                self.timed_next_state(Duration::from_millis(question.answer_time));
            }

            // Next state after awaiting answers is marking
            GameState::AwaitingAnswers => {
                self.mark_answers();
            }

            // Next state after marking is the next question
            GameState::Marked => {
                // Handle reaching the end of the questions
                if self.question_index + 1 >= self.config.questions.len() {
                    // Move to the finished state
                    self.set_state(GameState::Finished);
                    return;
                }

                // Increase the question index
                self.question_index += 1;
                self.question();
            }

            // Next state after finished is a reset game
            GameState::Finished => {
                self.reset_completely();
            }

            GameState::Stopped => {}
        }
    }

    /// Sends the provided server event to all the players
    /// and the host player
    ///
    /// # Arguments
    /// * event - The server event to send
    fn send_all(&self, event: ServerEvent) {
        // Wrap the message in an Arc to prevent cloning lots of heap data
        let event = Arc::new(event);

        // Send the message to all the players
        for player in &self.players {
            player.addr.send(event.clone());
        }

        // Send the message to the host
        self.host.addr.send(event);
    }

    /// Sets the current game state to the provided `state`. Emits a
    /// GameState event to all the listeners
    ///
    /// # Arguments
    /// * state - The state to set
    fn set_state(&mut self, state: GameState) {
        self.state = state;
        self.send_all(ServerEvent::GameState { state });
    }

    /// Completely resets all the game and player state to its initial values
    fn reset_completely(&mut self) {
        // Clear the task handle if present
        if let Some(task_handle) = self.task_handle.take() {
            task_handle.abort();
        }

        self.question_index = 0;

        self.players.iter_mut().for_each(|player| {
            // Reset the player answers
            player.answers.reset();
            // Reset the player score
            player.score = 0;
        });

        self.set_state(GameState::Lobby);
    }

    /// Updates the current state checking if all the players are ready
    /// then if they are progresses the state to [`GameState::AwaitingAnswers`]
    fn update_ready(&mut self) {
        // Ignore if we aren't expecting ready states
        if self.state != GameState::AwaitingReady {
            return;
        }

        // Check all players are ready
        let all_ready = self.players.iter().all(|player| player.ready) && self.host.ready;
        if !all_ready {
            return;
        }

        self.next_state()
    }

    /// Provides the current question to the all the players, updating
    /// the ready state and waiting for player readiness
    fn question(&mut self) {
        // Reset ready states for the players
        self.players
            .iter_mut()
            .for_each(|player| player.ready = false);

        // Reset host ready state
        self.host.ready = false;

        // Obtain the current question
        let question = self.config.questions[self.question_index].clone();

        // Send the question contents to the clients
        self.send_all(ServerEvent::Question { question });

        // Begin awaiting for ready messages
        self.set_state(GameState::AwaitingReady);
    }

    /// Marks all the answers provided by players, sends the scores and
    /// moves to the marked state
    fn mark_answers(&mut self) {
        // Get the current question
        let question = &self.config.questions[self.question_index];

        let scores: Vec<(SessionId, u32)> = self
            .players
            .iter_mut()
            .map(|player| {
                let answer = player.answers.get_answer(self.question_index);
                let score = answer.mark(question);

                // Increase the player score
                player.score += score.value();

                player.addr.send(Arc::new(ServerEvent::Score { score }));

                (player.id, player.score)
            })
            .collect();
        let scores = ScoreCollection(scores);

        // Update everyone's scores
        self.send_all(ServerEvent::Scores { scores });

        // Set state to marked
        self.set_state(GameState::Marked);
    }

    /// Obtains an image instance for the provided UUID
    ///
    /// # Arguments
    /// * uuid - The UUID of the image
    pub fn get_image(&self, uuid: Uuid) -> Option<Image> {
        self.config.images.get(&uuid).cloned()
    }

    /// Handles a player attempting to join this game
    ///
    /// # Arguments
    /// * id - The session ID of the joining player
    /// * addr - The player event target
    /// * name - The player desired name
    pub fn join(
        &mut self,
        id: SessionId,
        addr: EventTarget,
        name: String,
    ) -> Result<JoinedMessage, ServerError> {
        // Cannot join games that are already started or finished
        if !matches!(
            self.state,
            GameState::Lobby | GameState::Starting | GameState::Stopped
        ) {
            return Err(ServerError::NotJoinable);
        }

        // Trim name padding
        let name = name.trim();

        const MIN_NAME_LENGTH: usize = 1;
        const MAX_NAME_LENGTH: usize = 30;

        let name_length = name.len();
        if !(MIN_NAME_LENGTH..=MAX_NAME_LENGTH).contains(&name_length) {
            return Err(ServerError::InvalidNameLength);
        }

        // Name filtering
        if let Some(filter_type) = self.config.filtering.type_of()
            && name.is(filter_type)
        {
            return Err(ServerError::InappropriateName);
        }

        // Game already at max capacity
        if self.players.len() >= self.config.max_players {
            return Err(ServerError::CapacityReached);
        }

        // Error if username is already taken
        if self
            .players
            .iter()
            .any(|player| player.name.eq_ignore_ascii_case(name))
        {
            return Err(ServerError::UsernameTaken);
        }

        // Create the player
        let game_player = PlayerSession {
            id,
            addr,
            ready: false,

            name: Box::from(name),
            answers: PlayerAnswers::new(self.config.questions.len()),
            score: 0,
        };

        // Message sent to existing players for this player
        let joiner_message = Arc::new(ServerEvent::PlayerData {
            id: game_player.id,
            name: game_player.name.clone(),
        });

        // Notify all players of the existence of each other
        for player in &self.players {
            player.addr.send(joiner_message.clone());

            // Message describing the other player
            game_player.addr.send(Arc::new(ServerEvent::PlayerData {
                id: player.id,
                name: player.name.clone(),
            }));
        }

        // Notify the host of the join
        self.host.addr.send(joiner_message);

        self.players.push(game_player);

        Ok(JoinedMessage {
            token: self.token,
            config: self.config.clone(),
        })
    }

    /// Handles ready messages from a client by ID and updates
    /// the readiness accordingly
    ///
    /// # Arguments
    /// * id - The ID of the session that is ready
    pub fn ready(&mut self, id: SessionId) {
        if id == self.host.id {
            self.host.ready = true;
        } else {
            let player = self.players.iter_mut().find(|player| player.id == id);
            if let Some(player) = player {
                player.ready = true;
            }
        }

        self.update_ready();
    }

    /// Handles players providing answers, validates the answer
    /// is correct and handles advancing state onces all players
    /// have answered
    ///
    /// # Arguments
    /// * id - The session ID of the answering player
    /// * answer - The answer the player provided
    pub fn answer(&mut self, id: SessionId, answer: Answer) -> Result<(), ServerError> {
        let elapsed = self.start_time.elapsed();

        // Answers are not being accepted at the current time
        if self.state != GameState::AwaitingAnswers {
            return Err(ServerError::UnexpectedMessage);
        }

        let question = &self.config.questions[self.question_index];

        // Find the player within the game
        let player = self
            .players
            .iter_mut()
            .find(|player| player.id == id)
            .ok_or(ServerError::UnknownPlayer)?;

        // Ensure the answer is the right type of answer
        if !answer.is_valid(&question.data) {
            return Err(ServerError::InvalidAnswer);
        }

        // Set the player answer
        player
            .answers
            .set_answer(self.question_index, AnswerData { elapsed, answer });

        // If all the players have answered we can advance the state
        let all_answered = self
            .players
            .iter()
            .all(|player| player.answers.has_answer(self.question_index));

        if all_answered {
            self.next_state();
        }

        Ok(())
    }

    /// Handles player sending host actions
    ///
    /// # Arguments
    /// * id - The session ID of the player sending the action
    /// * action - The action the player sent
    pub fn host_action(&mut self, id: SessionId, action: HostAction) -> Result<(), ServerError> {
        // Handle messages that aren't from the game host
        if self.host.id != id {
            return Err(ServerError::InvalidPermission);
        }

        match action {
            HostAction::Reset => self.reset_completely(),
            HostAction::Next => self.next_state(),
        };

        Ok(())
    }

    /// Handles removing a player from the game, includes stopping the game when
    /// the host leaves
    ///
    /// # Arguments
    /// * id - The session ID of the player requesting the removal
    /// * target_id - The session ID of the player to remove
    /// * reason - The reason for removing the player
    pub fn remove_player(
        &mut self,
        id: SessionId,
        target_id: SessionId,
        mut reason: RemoveReason,
    ) -> Result<(), ServerError> {
        // Handle messages that aren't from the game host
        if target_id != id && self.host.id != id {
            return Err(ServerError::InvalidPermission);
        }

        // Host is removing itself (Game is stopping)
        if target_id == self.host.id {
            // Stop the game
            self.stop();
            return Ok(());
        }

        // Find the player position
        let index = self
            .players
            .iter()
            .position(|player| player.id == target_id)
            .ok_or(ServerError::UnknownPlayer)?;

        // Replace host remove reason for non hosts
        if RemoveReason::RemovedByHost == reason && id != self.host.id {
            reason = RemoveReason::Disconnected;
        }

        let kick_msg = Arc::new(ServerEvent::Kicked {
            id: target_id,
            reason,
        });

        // Inform each player of the removal
        self.players
            .iter()
            .for_each(|player| player.addr.send(kick_msg.clone()));

        // Inform the host of the player removal
        self.host.addr.send(kick_msg);

        // Remove the player
        self.players.remove(index);

        self.update_ready();

        // Reset the game if everyone disconnected while in progress
        if self.state != GameState::Finished && self.players.is_empty() {
            self.reset_completely();
        }

        Ok(())
    }

    /// Handles stopping the quiz, sends remove messages to all the players,
    /// removes from the games map and sets the state to stopped
    fn stop(&mut self) {
        // Don't try and stop the game twice
        if let GameState::Stopped = &self.state {
            return;
        }

        // Remove the game from the list of games
        tokio::spawn(Games::remove_game(self.token));

        // Tell all the players they've been kicked
        for player in &self.players {
            // Send the visual kick message
            player.addr.send(Arc::new(ServerEvent::Kicked {
                id: player.id,
                reason: RemoveReason::HostDisconnect,
            }));
        }

        self.host.addr.send(Arc::new(ServerEvent::Kicked {
            id: self.host.id,
            reason: RemoveReason::Disconnected,
        }));

        self.state = GameState::Stopped;

        debug!("Game stopped: {}", self.token);
    }
}

impl Drop for Game {
    fn drop(&mut self) {
        debug!("Game dropped: {}", self.token);
    }
}

/// Message containing the connected details for a connected player
pub struct JoinedMessage {
    /// The uniquely generated game token (e.g A3DLM)
    pub token: GameToken,
    /// Copy of the game configuration to send back
    pub config: Arc<GameConfig>,
}

/// Represents a session for the host player
struct HostSession {
    /// The ID of the referenced session
    id: SessionId,
    /// The addr to the session
    addr: EventTarget,
    /// The player ready state
    ready: bool,
}

impl HostSession {
    pub fn new(id: SessionId, addr: EventTarget) -> Self {
        Self {
            id,
            addr,
            ready: false,
        }
    }
}

/// Represents a session and associated data
/// for a player within a quiz
struct PlayerSession {
    /// The ID of the referenced session
    id: SessionId,
    /// The addr to the session
    addr: EventTarget,
    /// The player ready state
    ready: bool,

    /// The player name
    name: ImStr,
    /// The players answers and the score they got for them
    answers: PlayerAnswers,
    /// The player total score
    score: u32,
}

/// Structure storing the player answers. Fixed length to
/// the total number of questions in the game
struct PlayerAnswers {
    /// The actual player answers
    values: Box<[PlayerAnswer]>,
}

impl PlayerAnswers {
    /// Creates a new player answers structure of the
    /// provided length
    ///
    /// # Arguments
    /// * length - The length of the answers
    fn new(length: usize) -> Self {
        // Create all the answers collecting into the boxed slice
        let values: Box<[PlayerAnswer]> = (0..length).map(|_| PlayerAnswer::default()).collect();
        Self { values }
    }

    /// Resets the state of each player answer replacing the
    /// score and answer data with None
    fn reset(&mut self) {
        self.values.iter_mut().for_each(|value| {
            value.data = None;
            value.score = None;
        })
    }

    /// Sets the player answer at the provided index to the
    /// provided value
    ///
    /// # Arguments
    /// * index - The index of the answer within the values array
    /// * answer - The answer to set the value to
    fn set_answer(&mut self, index: usize, answer: AnswerData) {
        debug_assert!(index < self.values.len());
        self.values[index].data = Some(answer);
    }

    /// Provides mutable access to the player answer at the provided
    /// index
    ///
    /// # Arguments
    /// * index - The index of the answer within the values array
    fn get_answer(&mut self, index: usize) -> &mut PlayerAnswer {
        debug_assert!(index < self.values.len());
        &mut self.values[index]
    }

    /// Provides a borrow to the player answer at the provided
    /// index
    ///
    /// # Arguments
    /// * index - The index of the answer within the values array
    fn get_answer_ref(&self, index: usize) -> &PlayerAnswer {
        debug_assert!(index < self.values.len());
        &self.values[index]
    }

    /// Checks if theres an answer stored at the provided index
    ///
    /// # Arguments
    /// * index - The index of the answer within the values array
    fn has_answer(&self, index: usize) -> bool {
        debug_assert!(index < self.values.len());
        self.values[index].data.is_some()
    }
}

/// Structure storing a player answer and the score provided
/// for it
#[derive(Default)]
struct PlayerAnswer {
    /// The answer provided by the player
    data: Option<AnswerData>,
    /// The score provided by the server
    score: Option<Score>,
}

impl PlayerAnswer {
    /// Marks the current question updating the stored score
    /// and returns the score
    ///
    /// # Arguments
    /// * question - The question to mark this answer against
    fn mark(&mut self, question: &Question) -> Score {
        let score = self.mark_impl(question);
        self.score = Some(score);
        score
    }

    /// Marking implementation which marks the current answer
    /// using the provided question as the correct answers.
    ///
    /// # Arguments
    /// * question - The question to mark this answer against
    fn mark_impl(&self, question: &Question) -> Score {
        let answer = match &self.data {
            Some(value) => value,
            None => return Score::Incorrect,
        };

        let elapsed_ms = answer.elapsed.as_millis() as u32;
        let is_bonus = elapsed_ms <= question.bonus_score_time;

        // Calculate the % amount between the min and max answer times
        let answer_time_percent = 1.0 - ((elapsed_ms as f32) / (question.answer_time as f32));

        let scoring = &question.scoring;

        // The base score from the answer time and the bonus
        let mut base_score = scoring.min_score
            + ((scoring.max_score - scoring.min_score) as f32 * answer_time_percent) as u32;

        // Append bonus score amount
        if is_bonus {
            base_score += scoring.bonus_score;
        }

        use Answer as A;
        use QuestionData as Q;

        match (&answer.answer, &question.data) {
            (A::Single { answer }, Q::Single { answers, .. }) => {
                Self::mark_single(*answer, answers, base_score)
            }
            (A::Multiple { answers: indexes }, Q::Multiple { answers, .. }) => {
                Self::mark_multiple(indexes, answers, base_score)
            }
            (A::TrueFalse { answer }, Q::TrueFalse { answer: actual }) => {
                Self::mark_bool(*answer, *actual, base_score)
            }
            (
                A::Typer { answer },
                Q::Typer {
                    answers,
                    ignore_case,
                },
            ) => Self::mark_typer(answer, answers, *ignore_case, base_score),
            // Mismatched types shouldn't be possible but
            // will be marked as incorrect
            _ => Score::Incorrect,
        }
    }

    /// Marks a single choice question
    ///
    /// # Arguments
    /// * answer - The index of the users answer
    /// * answers - The answers for the question
    /// * base_score - The base score for correct answers
    fn mark_single(answer: usize, answers: &[AnswerValue], base_score: u32) -> Score {
        let is_valid = answers
            .get(answer)
            .map(|value| value.correct)
            .unwrap_or(false);
        if is_valid {
            Score::Correct { value: base_score }
        } else {
            Score::Incorrect
        }
    }

    /// Marks a multiple choice question
    ///
    /// # Arguments
    /// * indexes - The indexes of the answers the player chose
    /// * answers - The answers for the question
    /// * base_score - The base score for correct answers
    fn mark_multiple(indexes: &[usize], answers: &[AnswerValue], base_score: u32) -> Score {
        let count_answers = indexes.len();

        // The total number of actual correct answers
        let count_expected = answers.iter().filter(|value| value.correct).count();

        // Didn't provide enough answer or provided too many
        if count_answers < 1 || count_answers > count_expected {
            return Score::Incorrect;
        }

        // Count the number of provided correct answers
        let count_correct = indexes
            .iter()
            .filter_map(|index| answers.get(*index))
            .filter(|value| value.correct)
            .count();

        if count_correct < 1 {
            Score::Incorrect
        } else if count_correct == count_expected {
            Score::Correct { value: base_score }
        } else {
            // % correct out of total answers
            let percent = count_correct as f32 / count_expected as f32;
            let score = ((base_score as f32) * percent).round() as u32;
            Score::Partial {
                value: score,
                count: count_correct as u32,
                total: count_expected as u32,
            }
        }
    }

    /// Marks a True / False boolean question
    ///
    /// # Arguments
    /// * answer - The boolean answer the player chose
    /// * actual - The correct answer for the question
    /// * base_score - The base score for correct answers
    fn mark_bool(answer: bool, actual: bool, base_score: u32) -> Score {
        if answer == actual {
            Score::Correct { value: base_score }
        } else {
            Score::Incorrect
        }
    }

    /// Marks a typing question
    ///
    /// # Arguments
    /// * answer - The player typed answer
    /// * answers - The question valid answers
    /// * ignore_case - Whether to ignore case when matching
    /// * base_score - The base score for correct answers
    fn mark_typer(answer: &str, answers: &[ImStr], ignore_case: bool, base_score: u32) -> Score {
        // Trim extra whitespace
        let answer = answer.trim();

        let equal_fn = if ignore_case {
            str::eq_ignore_ascii_case
        } else {
            str::eq
        };

        let correct = answers.iter().any(|value| equal_fn(answer, value));

        if correct {
            Score::Correct { value: base_score }
        } else {
            Score::Incorrect
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        game::PlayerAnswer,
        types::{AnswerValue, Score},
    };

    pub const TEST_CORRECT_SCORE: u32 = 100;

    #[test]
    fn test_mark_single_correct() {
        let answers = &[
            AnswerValue {
                value: "Test".into(),
                correct: true,
            },
            AnswerValue {
                value: "Test".into(),
                correct: false,
            },
        ];

        let score = PlayerAnswer::mark_single(0, answers, TEST_CORRECT_SCORE);
        let expected = Score::Correct {
            value: TEST_CORRECT_SCORE,
        };
        assert_eq!(score, expected);
    }

    #[test]
    fn test_mark_single_incorrect() {
        let answers = &[
            AnswerValue {
                value: "Test".into(),
                correct: true,
            },
            AnswerValue {
                value: "Test".into(),
                correct: false,
            },
        ];

        let score = PlayerAnswer::mark_single(1, answers, TEST_CORRECT_SCORE);
        let expected = Score::Incorrect;

        assert_eq!(score, expected);
    }

    #[test]
    fn test_mark_multiple_correct() {
        let answers = &[
            AnswerValue {
                value: "Test".into(),
                correct: true,
            },
            AnswerValue {
                value: "Test".into(),
                correct: false,
            },
            AnswerValue {
                value: "Test".into(),
                correct: true,
            },
            AnswerValue {
                value: "Test".into(),
                correct: false,
            },
        ];

        let score = PlayerAnswer::mark_multiple(&[0, 2], answers, TEST_CORRECT_SCORE);
        let expected = Score::Correct {
            value: TEST_CORRECT_SCORE,
        };

        assert_eq!(score, expected);
    }

    #[test]
    fn test_mark_multiple_partial() {
        let answers = &[
            AnswerValue {
                value: "Test".into(),
                correct: true,
            },
            AnswerValue {
                value: "Test".into(),
                correct: false,
            },
            AnswerValue {
                value: "Test".into(),
                correct: true,
            },
            AnswerValue {
                value: "Test".into(),
                correct: false,
            },
        ];

        let score = PlayerAnswer::mark_multiple(&[1, 2], answers, TEST_CORRECT_SCORE);
        let expected = Score::Partial {
            value: TEST_CORRECT_SCORE / 2,
            count: 1,
            total: 2,
        };

        assert_eq!(score, expected);
    }

    #[test]
    fn test_mark_multiple_incorrect() {
        let answers = &[
            AnswerValue {
                value: "Test".into(),
                correct: true,
            },
            AnswerValue {
                value: "Test".into(),
                correct: false,
            },
            AnswerValue {
                value: "Test".into(),
                correct: true,
            },
            AnswerValue {
                value: "Test".into(),
                correct: false,
            },
        ];

        let score = PlayerAnswer::mark_multiple(&[1, 3], answers, TEST_CORRECT_SCORE);
        let expected = Score::Incorrect;

        assert_eq!(score, expected);
    }

    #[test]
    fn test_mark_bool_correct() {
        let score = PlayerAnswer::mark_bool(true, true, TEST_CORRECT_SCORE);
        let expected = Score::Correct {
            value: TEST_CORRECT_SCORE,
        };

        assert_eq!(score, expected);
    }

    #[test]
    fn test_mark_bool_incorrect() {
        let score = PlayerAnswer::mark_bool(true, false, TEST_CORRECT_SCORE);
        let expected = Score::Incorrect;

        assert_eq!(score, expected);
    }

    #[test]
    fn test_mark_typer_correct_case_sensitive() {
        let score = PlayerAnswer::mark_typer(
            "Test",
            &["Test".into(), "Test 1".into(), "Test 2".into()],
            false,
            TEST_CORRECT_SCORE,
        );
        let expected = Score::Correct {
            value: TEST_CORRECT_SCORE,
        };

        assert_eq!(score, expected);
    }

    #[test]
    fn test_mark_typer_correct_case_insensitive() {
        let score = PlayerAnswer::mark_typer(
            "test",
            &["Test".into(), "Test 1".into(), "Test 2".into()],
            true,
            TEST_CORRECT_SCORE,
        );
        let expected = Score::Correct {
            value: TEST_CORRECT_SCORE,
        };

        assert_eq!(score, expected);
    }

    #[test]
    fn test_mark_typer_incorrect_case_sensitive() {
        let score = PlayerAnswer::mark_typer(
            "test",
            &["Test".into(), "Test 1".into(), "Test 2".into()],
            false,
            TEST_CORRECT_SCORE,
        );
        let expected = Score::Incorrect;

        assert_eq!(score, expected);
    }

    #[test]
    fn test_mark_typer_incorrect_case_insensitive() {
        let score = PlayerAnswer::mark_typer(
            "test 5",
            &["Test".into(), "Test 1".into(), "Test 2".into()],
            true,
            TEST_CORRECT_SCORE,
        );
        let expected = Score::Incorrect;

        assert_eq!(score, expected);
    }
}
