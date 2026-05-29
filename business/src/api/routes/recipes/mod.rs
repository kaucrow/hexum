pub mod dtos;
pub mod sync;
pub mod search;
pub mod get_by_id;

pub use sync::sync;
pub use search::search;
pub use get_by_id::get_by_id;

use crate::{
    prelude::*,
    api::*,
    features::*,
};

impl From<recipe::UseCaseError> for ApiError {
    fn from(e: recipe::UseCaseError) -> Self {
        #[allow(unreachable_patterns)]
        match e {
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