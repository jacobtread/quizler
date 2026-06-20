use crate::{
    game::{Game, GameRef, config::GameConfig},
    session::{EventTarget, SessionId},
    types::{GameToken, ServerError},
};
use parking_lot::RwLock;
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
    time::Duration,
};
use tokio::{task::AbortHandle, time::sleep};
use uuid::Uuid;

/// Global instance for storing games
static GAMES: LazyLock<RwLock<Games>> = LazyLock::new(Default::default);

/// Central store for storing all the references to the individual
/// games that are currently running
#[derive(Default)]
pub struct Games {
    /// Map of the game tokens to the actual game itself
    games: HashMap<GameToken, GameRef>,
    /// Map of UUID's to game configurations that are preparing to start
    preparing: HashMap<Uuid, PreparingGame>,
}

/// Game state for a game thats been created from the HTTP
/// API but hasn't yet been initialized by a host socket
struct PreparingGame {
    /// The config being prepared
    config: GameConfig,
    /// Handle to the expiry task to abort the expiry timeout
    expiry_handle: AbortHandle,
}

/// Message containing the details of a game that has been successfully
/// connected to by the host (The game has finished being prepared)
pub struct InitializedMessage {
    /// The uniquely generated game token (e.g A3DLM)
    pub token: GameToken,
    /// The full game config to be used while playing
    pub config: Arc<GameConfig>,
    /// Reference to the created game
    pub game: GameRef,
}

impl Games {
    /// Obtains a static reference to the global
    /// games store
    fn global() -> &'static RwLock<Games> {
        &GAMES
    }

    /// Prepares a new Quiz for creation. Stores the uploaded config
    /// with a UUID provided the UUID for the host to connect with
    ///
    /// # Arguments
    /// * config - The config for the quiz
    pub fn prepare(config: GameConfig) -> Uuid {
        const GAME_EXPIRY_TIME: Duration = Duration::from_secs(60 * 10);

        let id = Uuid::new_v4();

        // Create an expiry timeout task that removes the preparing game automatically
        // unless cancelled before the expiry time
        let expiry_handle = tokio::spawn(async move {
            sleep(GAME_EXPIRY_TIME).await;
            let mut games = Self::global().write();
            games.preparing.remove(&id);
        })
        .abort_handle();

        let mut games = Self::global().write();
        games.preparing.insert(
            id,
            PreparingGame {
                config,
                expiry_handle,
            },
        );

        id
    }

    /// Initializes a prepared game using the provided host details and
    /// prepare config UUID
    ///
    /// # Arguments
    /// * uuid - The UUID of the prepared config
    /// * host_id - The session ID of the host player
    /// * host_target - The event target for the host player
    pub fn initialize(
        uuid: Uuid,
        host_id: SessionId,
        host_target: EventTarget,
    ) -> Result<InitializedMessage, ServerError> {
        // Write lock is required for updating state
        let mut games = Self::global().write();

        // Consume the provided prepared config
        let prepared = games
            .preparing
            .remove(&uuid)
            .ok_or(ServerError::InvalidToken)?;

        // Abort the expiry task
        prepared.expiry_handle.abort();

        // Create a new game token
        let token = GameToken::unique_token(&games.games);

        // Create the game
        let config = Arc::new(prepared.config);
        let game = Game::new(token, host_id, host_target, config.clone());
        let game = Arc::new(RwLock::new(game));

        // Insert the game into the games map
        games.games.insert(token, game.clone());

        Ok(InitializedMessage {
            token,
            config,
            game,
        })
    }

    /// Obtains a cloned Arc for a game with the specific token
    /// if one exists
    ///
    /// # Arguments
    /// * token - The token of the game to get
    pub fn get_game(token: &GameToken) -> Option<GameRef> {
        Self::global().read().games.get(token).cloned()
    }

    /// Removes the game with the provided [`GameToken`] from
    /// the map of games
    pub fn remove_game(token: GameToken) {
        Self::global().write().games.remove(&token);
    }
}
