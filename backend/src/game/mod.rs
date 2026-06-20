use crate::{
    game::{
        config::GameConfig,
        manager::Games,
        player::{GameSession, GameSessionMut},
    },
    msg::ServerEvent,
    session::{EventTarget, SessionId},
    types::{
        Answer, AnswerData, GameToken, HostAction, Image, RemoveReason, ScoreCollection,
        ServerError,
    },
};
use answer::PlayerAnswers;
use log::debug;
use player::{HostSession, PlayerSession};
use serde::Serialize;
use std::{
    ops::{Add, Sub},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{task::AbortHandle, time::sleep};
use uuid::Uuid;

mod answer;
pub mod config;
pub mod manager;
pub mod player;

/// Reference to a game behind an Arc and a RwLock
pub type GameRef = Arc<parking_lot::RwLock<Game>>;

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
    task: Option<TaskState>,
    /// Start time updated for each question
    start_time: Instant,
}

struct TaskState {
    /// Handle to the task transition state
    handle: AbortHandle,
    /// The expected instant of when the task should complete
    expected_completion_at: Instant,
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

impl GameState {
    pub fn is_joinable(&self) -> bool {
        matches!(
            self,
            GameState::Lobby | GameState::Starting | GameState::Stopped
        )
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
            task: None,
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

        let now = Instant::now();
        let expected_completion_at = now.add(duration);
        let handle = tokio::spawn(async move {
            sleep(duration).await;
            let game = Games::get_game(&token);
            if let Some(game) = game {
                let lock = &mut *game.write();
                lock.task = None;
                lock.next_state();
            }
        });

        self.task = Some(TaskState {
            handle: handle.abort_handle(),
            expected_completion_at,
        });

        // Send timer message with the duration time
        self.send_all(ServerEvent::Timer {
            value: duration.as_millis() as u32,
        });
    }

    /// Find a session within the game using the session ID
    /// could be a player or the host itself
    fn find_session(&self, session_id: SessionId) -> Option<GameSession<'_>> {
        if self.host.id == session_id {
            Some(GameSession::Host(&self.host))
        } else {
            self.players
                .iter()
                .find(|player| player.id == session_id)
                .map(GameSession::Player)
        }
    }
    /// Find a session within the game using the session ID
    /// could be a player or the host itself
    fn find_session_mut(&mut self, session_id: SessionId) -> Option<GameSessionMut<'_>> {
        if self.host.id == session_id {
            Some(GameSessionMut::Host(&mut self.host))
        } else {
            self.players
                .iter_mut()
                .find(|player| player.id == session_id)
                .map(GameSessionMut::Player)
        }
    }

    /// Creates a new player answers set based on the required
    /// number of question slots
    fn create_player_answers(&self) -> PlayerAnswers {
        PlayerAnswers::new(self.config.questions.len())
    }

    /// Creates a [ScoreCollection] of all the current player scores
    fn player_scores_collection(&self) -> ScoreCollection {
        let scores: Vec<(SessionId, u32)> = self
            .players
            .iter()
            .map(|player| (player.id, player.score))
            .collect();
        ScoreCollection(scores)
    }

    /// Handle resuming the game session for the player by `session_id` sends the
    /// relevant information to get the player up to speed
    pub fn resume_player(&self, session_id: SessionId) {
        let session = match self.find_session(session_id) {
            Some(value) => value,
            None => return,
        };

        let target_addr = session.addr();

        // Notify the player they have resumed a game
        target_addr.send(Arc::new(ServerEvent::ResumedGame {
            id: session_id,
            host: matches!(&session, GameSession::Host(_)),
            token: self.token,
            config: self.config.clone(),
            name: session.player().as_ref().map(|player| player.name.clone()),
        }));

        // Notify the player of all the players in the game (Including themselves)
        for player in &self.players {
            target_addr.send_owned(player.introduction_event());
        }

        // Update everyone's scores
        // (Must come before game state so that the frontend can compute finished scores)
        let scores = self.player_scores_collection();
        target_addr.send_owned(ServerEvent::Scores { scores });

        // Send the player the current game state
        target_addr.send_owned(ServerEvent::GameState { state: self.state });

        // If a timed task is actively ongoing compute the remaining time and send
        // it to the user
        if let Some(task) = self.task.as_ref() {
            let now = Instant::now();
            let duration = task.expected_completion_at.sub(now);
            target_addr.send_owned(ServerEvent::Timer {
                value: duration.as_millis() as u32,
            });
        }

        match self.state {
            // Send the current question to the player for states that depend on one
            GameState::Starting
            | GameState::AwaitingReady
            | GameState::PreQuestion
            | GameState::Marked => {
                let question = self.config.questions[self.question_index].clone();
                target_addr.send_owned(ServerEvent::Question { question });

                // Provide the answer score if the question has been scored
                // (Only for players and not the host)
                if let Some(target_player) = session.player() {
                    let answer = target_player.answers.get_answer_ref(self.question_index);
                    if let Some(score) = answer.score() {
                        target_addr.send_owned(ServerEvent::Score { score: *score });
                    }
                }
            }

            _ => {}
        }
    }

    /// Moves the game to the next state based on its current state
    fn next_state(&mut self) {
        // Cancel a delayed task if one is running
        if let Some(task_handle) = self.task.take() {
            task_handle.handle.abort();
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
        if let Some(task_handle) = self.task.take() {
            task_handle.handle.abort();
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

    fn reset_readiness(&mut self) {
        // Reset ready states for the players
        self.players
            .iter_mut()
            .for_each(|player| player.ready = false);

        // Reset host ready state
        self.host.ready = false;
    }

    /// Provides the current question to the all the players, updating
    /// the ready state and waiting for player readiness
    fn question(&mut self) {
        self.reset_readiness();

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

    /// Checks if the game is at its capacity
    fn is_at_capacity(&self) -> bool {
        self.players.len() >= self.config.max_players
    }

    /// Checks if a name is taken by any other players in the game
    /// (Case insensitive check)
    fn is_name_taken(&self, name: &str) -> bool {
        self.players
            .iter()
            .any(|player| player.name.eq_ignore_ascii_case(name))
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
        if !self.state.is_joinable() {
            return Err(ServerError::NotJoinable);
        }

        // Trim name padding
        let name = self.config.validate_name(&name)?;

        // Game already at max capacity
        if self.is_at_capacity() {
            return Err(ServerError::CapacityReached);
        }

        // Error if username is already taken
        if self.is_name_taken(name.as_ref()) {
            return Err(ServerError::UsernameTaken);
        }

        // Create the player
        let game_player = PlayerSession::new(id, addr, name, self.create_player_answers());
        self.introduce_player(&game_player);
        self.players.push(game_player);

        Ok(JoinedMessage {
            token: self.token,
            config: self.config.clone(),
        })
    }

    /// Announces a `player` to all other players in the game and all other
    /// players in the game to `player`.
    ///
    /// Used when the player first joins a game to notify everyone of
    /// each others existence
    fn introduce_player(&self, player: &PlayerSession) {
        // Message sent to existing players for this player
        let joiner_message = Arc::new(player.introduction_event());

        // Notify all players of the existence of each other
        for other_player in &self.players {
            other_player.addr.send(joiner_message.clone());
            player.addr.send_owned(other_player.introduction_event());
        }

        // Notify the player of themselves
        player.addr.send(joiner_message.clone());

        // Notify the host of the join
        self.host.addr.send(joiner_message);
    }

    /// Handles ready messages from a client by ID and updates
    /// the readiness accordingly
    ///
    /// # Arguments
    /// * id - The ID of the session that is ready
    pub fn ready(&mut self, id: SessionId) {
        if let Some(mut session) = self.find_session_mut(id) {
            session.mark_ready();
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

        // Notify everyone of the kicks
        self.send_all(ServerEvent::Kicked {
            id: target_id,
            reason,
        });

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
        Games::remove_game(self.token);

        // Tell all the players they've been kicked
        for player in &self.players {
            // Send the visual kick message
            player.addr.send_owned(ServerEvent::Kicked {
                id: player.id,
                reason: RemoveReason::HostDisconnect,
            });
        }

        self.host.addr.send_owned(ServerEvent::Kicked {
            id: self.host.id,
            reason: RemoveReason::Disconnected,
        });

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
