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
            auth::UseCaseError::InvalidPassword
            | auth::UseCaseError::InvalidCredentials => {
                warn!("Invalid credentials: {e}");
                ApiError::Unauthorized("Invalid username/email or password.".to_string())
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
                ApiError::Unauthorized("Invalid username/email or password.".to_string())
            }
            auth::UseCaseError::UserInactive => {
                warn!("User is inactive: {e}");
                ApiError::Unauthorized("Invalid username/email or password.".to_string())
            }
            auth::UseCaseError::UserNotVerified => {
                warn!("User email has not been verified: {e}");
                ApiError::Unauthorized("Invalid username/email or password.".to_string())
            }
            auth::UseCaseError::TooManyRequests { retry_after_secs } => {
                warn!("Rate limit exceeded, retry after {retry_after_secs}s");
                ApiError::TooManyRequests(
                    "Too many requests. Please slow down.".to_string(),
                    Some(retry_after_secs),
                )
            }
            auth::UseCaseError::LockedOut { retry_after_secs } => {
                warn!("Account locked out, retry after {retry_after_secs}s");
                ApiError::TooManyRequests(
                    "Account temporarily locked due to too many failed attempts.".to_string(),
                    Some(retry_after_secs),
                )
            }
            auth::UseCaseError::Parse(s) => {
                warn!("Parse error: {s}");
                ApiError::BadRequest(s)
            }
            auth::UseCaseError::Internal(e) => {
                error!("An internal error occurred: {e}");
                ApiError::Internal
            }
        }
    }
}