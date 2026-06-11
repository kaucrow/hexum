pub mod dtos;
pub mod local;
pub mod oauth;
pub mod refresh;
pub mod logout;

pub use refresh::refresh_session;
pub use logout::logout;

use crate::{
    prelude::*,
    features::auth,
    api::ApiError,
};

impl From<auth::UseCaseError> for ApiError {
    fn from(e: auth::UseCaseError) -> Self {
        match e {
            auth::UseCaseError::InvalidPassword => {
                warn!("Invalid password: {e}");
                ApiError::Unauthorized("No user with these credentials was found.".to_string())
            }
            auth::UseCaseError::InvalidAccessToken(s) => {
                warn!("Invalid access token: {s}");
                ApiError::Unauthorized("The access token is invalid or expired.".to_string())
            }
            auth::UseCaseError::InvalidRefreshToken => {
                warn!("Invalid refresh token: {e}");
                ApiError::Unauthorized("The refresh token is invalid or expired.".to_string())
            }
            auth::UseCaseError::InvalidOAuthCode(s) => {
                warn!("Invalid OAuth code: {s}");
                ApiError::BadRequest(s)
            }
            auth::UseCaseError::UserNotFound => {
                warn!("User not found: {e}");
                ApiError::Unauthorized("No user with these credentials was found.".to_string())
            }
            auth::UseCaseError::UserInactive => {
                warn!("User is inactive: {e}");
                ApiError::Unauthorized("The user has been disabled.".to_string())
            }
            auth::UseCaseError::UserNotVerified => {
                warn!("User email has not been verified: {e}");
                ApiError::Unauthorized("The user email has not been verified.".to_string())
            }
            auth::UseCaseError::Parse(s) => {
                warn!("Parse error: {s}");
                ApiError::BadRequest(s)
            }
            auth::UseCaseError::Internal(e) => {
                error!("An internal error occurred: {e}");
                ApiError::Internal("An internal error occurred.".to_string())
            }
        }
    }
}