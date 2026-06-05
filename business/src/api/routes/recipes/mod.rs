pub mod dtos;
pub mod sync;
pub mod search;
pub mod get_by_id;
pub mod explore;
pub mod create;
pub mod history;
pub mod delete;
pub mod created;

pub use sync::sync;
pub use search::search;
pub use get_by_id::get_by_id;
pub use explore::{popular, latest, top_tags};
pub use create::create;
pub use history::history;
pub use delete::delete;
pub use created::created;

use std::collections::HashMap;

use crate::{
    prelude::*,
    api::*,
    features::*,
};

impl From<recipe::UseCaseError> for ApiError {
    fn from(e: recipe::UseCaseError) -> Self {
        #[allow(unreachable_patterns)]
        match e {
            recipe::UseCaseError::MissingSearchParams => {
                warn!("Tried to search recipe without query or tags");
                let mut errors = HashMap::new();
                errors.insert("query".to_string(), vec![e.to_string()]);
                errors.insert("tags".to_string(), vec![e.to_string()]);
                ApiError::Validation(errors)
            }
            recipe::UseCaseError::EmptyName => {
                warn!("Tried to create a recipe with an empty name");
                let mut errors = HashMap::new();
                errors.insert("name".to_string(), vec![e.to_string()]);
                ApiError::Validation(errors)
            }
            recipe::UseCaseError::EmptyInstructions => {
                warn!("Tried to create a recipe with empty instructions");
                let mut errors = HashMap::new();
                errors.insert("instructions".to_string(), vec![e.to_string()]);
                ApiError::Validation(errors)
            }
            recipe::UseCaseError::Internal(e) => {
                error!("An internal error occurred: {e}");
                ApiError::Internal("An internal error occurred".to_string())
            }
            _ => {
                error!("Unexpected domain error: {e}");
                ApiError::Internal("An internal error occurred".to_string())
            }
        }
    }
}