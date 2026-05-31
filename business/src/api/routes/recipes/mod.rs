pub mod dtos;
pub mod sync;
pub mod search;
pub mod get_by_id;

pub use sync::sync;
pub use search::search;
pub use get_by_id::get_by_id;

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