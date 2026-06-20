use std::{collections::HashMap, sync::Arc};

use rustrict::CensorStr;
use serde::Serialize;

use crate::types::{ImStr, Image, ImageRef, NameFiltering, Question, ServerError};

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

    pub fn validate_name(&self, name: &str) -> Result<ImStr, ServerError> {
        const MIN_NAME_LENGTH: usize = 1;
        const MAX_NAME_LENGTH: usize = 30;

        let name = name.trim();
        let name_length = name.len();
        if !(MIN_NAME_LENGTH..=MAX_NAME_LENGTH).contains(&name_length) {
            return Err(ServerError::InvalidNameLength);
        }

        // Name filtering
        if let Some(filter_type) = self.filtering.type_of()
            && name.is(filter_type)
        {
            return Err(ServerError::InappropriateName);
        }

        Ok(Box::from(name))
    }
}
