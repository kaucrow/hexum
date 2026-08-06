pub mod dtos;
pub mod list_users;
pub mod requests;

pub use list_users::*;
pub use requests::*;

use crate::{
    prelude::*,
    api::*,
    features::friends::{self, UserSummary, FriendRequest},
};
use self::dtos::*;

// ─── DTO mappings ───

impl From<UserSummary> for UserSummaryResponse {
    fn from(u: UserSummary) -> Self {
        Self {
            id: u.id.to_string(),
            username: u.username,
            profile_picture_url: u.profile_picture_url,
        }
    }
}

impl From<FriendRequest> for FriendRequestResponse {
    fn from(r: FriendRequest) -> Self {
        Self {
            id: r.id.to_string(),
            sender_id: r.sender_id.to_string(),
            sender_username: r.sender_username,
            receiver_id: r.receiver_id.to_string(),
            receiver_username: r.receiver_username,
            status: r.status.as_str().to_string(),
            created_at: r.created_at.to_rfc3339(),
            updated_at: r.updated_at.to_rfc3339(),
        }
    }
}

// ─── Error mappings ───

impl From<friends::UseCaseError> for ApiError {
    fn from(e: friends::UseCaseError) -> Self {
        match e {
            friends::UseCaseError::CannotFriendYourself => {
                ApiError::BadRequest(e.to_string())
            }
            friends::UseCaseError::PendingRequestExists => {
                ApiError::Conflict(e.to_string())
            }
            friends::UseCaseError::RequestNotFound => {
                ApiError::NotFound(e.to_string())
            }
            friends::UseCaseError::NotReceiver => {
                ApiError::BadRequest(e.to_string())
            }
            friends::UseCaseError::Internal(msg) => {
                error!("Internal friends error: {msg}");
                ApiError::Internal("An internal error occurred".to_string())
            }
        }
    }
}