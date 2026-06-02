pub mod dtos;
pub mod get_groups;
pub mod get_recipes;
pub mod create;
pub mod add_recipe;
pub mod remove_recipe;

pub use get_groups::get_groups;
pub use get_recipes::get_recipes;
pub use create::create;
pub use add_recipe::add_recipe;
pub use remove_recipe::remove_recipe;

use crate::{
    prelude::*,
    api::*,
    features::*,
};

impl From<group::UseCaseError> for ApiError {
    fn from(e: group::UseCaseError) -> Self {
        #[allow(unreachable_patterns)]
        match e {
            group::UseCaseError::GroupNotFound => {
                warn!("Tried to access a non-existent group");
                ApiError::NotFound(e.to_string())
            }
            group::UseCaseError::NotGroupOwner => {
                warn!("User tried to access a group they don't own");
                ApiError::Unauthorized(e.to_string())
            }
            group::UseCaseError::EmptyName => {
                warn!("Tried to create a group with an empty name");
                let mut errors = HashMap::new();
                errors.insert("name".to_string(), vec![e.to_string()]);
                ApiError::Validation(errors)
            }
            group::UseCaseError::Internal(e) => {
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