pub mod dtos;
pub mod get;
pub mod send;

pub use get::*;
pub use send::*;

use crate::{
    prelude::*,
    api::*,
    features::messages,
};
use self::dtos::*;

// ─── DTO mappings ───

impl From<messages::Message> for MessageResponse {
    fn from(m: messages::Message) -> Self {
        Self {
            id: m.id.to_string(),
            sender_id: m.sender_id.to_string(),
            receiver_id: m.receiver_id.to_string(),
            r#type: m.message_type.as_str().to_string(),
            content: m.content,
            created_at: m.created_at.to_rfc3339(),
        }
    }
}

// ─── Error mappings ───

impl From<messages::UseCaseError> for ApiError {
    fn from(e: messages::UseCaseError) -> Self {
        match e {
            messages::UseCaseError::NotFriends => {
                ApiError::Forbidden(e.to_string())
            }
            messages::UseCaseError::CannotMessageYourself => {
                ApiError::BadRequest(e.to_string())
            }
            messages::UseCaseError::Internal(msg) => {
                error!("Internal messages error: {msg}");
                ApiError::Internal("An internal error occurred".to_string())
            }
        }
    }
}
