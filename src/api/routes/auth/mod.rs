pub mod local;
pub mod oauth;
pub mod refresh;
pub mod logout;
pub mod dtos;

pub use refresh::refresh_session;
pub use logout::logout;

use axum_extra::extract::cookie::{Cookie, SameSite};
use time::Duration;

use crate::{
    config::ApiProtocol,
    prelude::*,
    features::auth,
    api::ApiError,
};

// Helper function to build cookies
fn build_cookie<'a>(name: &'a str, value: String, path: &'a str, protocol: &ApiProtocol) -> Cookie<'a> {
    let mut cookie = Cookie::build((name, value))
        .http_only(true)
        .same_site(SameSite::Strict)
        .path(path);

    if matches!(protocol, ApiProtocol::Http) {
        cookie = cookie.secure(false);
    } else {
        cookie = cookie.secure(true);
    }

    cookie.build()
}

// Helper function to build removal cookies
fn build_removal_cookie<'a>(name: &'a str, path: &'a str, protocol: &ApiProtocol) -> Cookie<'a> {
    let mut cookie = Cookie::build((name, ""))
        .http_only(true)
        .same_site(SameSite::Strict)
        .path(path)
        .max_age(Duration::ZERO);

    if matches!(protocol, ApiProtocol::Http) {
        cookie = cookie.secure(false);
    } else {
        cookie = cookie.secure(true);
    }

    cookie.build()
}

impl From<auth::UseCaseError> for ApiError {
    fn from(e: auth::UseCaseError) -> Self {
        #[allow(unreachable_patterns)]
        match e {
            auth::UseCaseError::InvalidPassword => {
                warn!("Invalid password: {e}");
                ApiError::Unauthorized("No user with these credentials was found.".to_string())
            }
            auth::UseCaseError::UserNotFound => {
                warn!("User not found : {e}");
                ApiError::Unauthorized("No user with these credentials was found.".to_string())
            }
            auth::UseCaseError::InvalidRefreshToken => {
                warn!("Invalid refresh token: {e}");
                ApiError::Unauthorized("The refresh token is invalid or expired.".to_string())
            }
            auth::UseCaseError::UserInactive => {
                warn!("User is inactive: {e}");
                ApiError::Unauthorized("The user has been disabled.".to_string())
            }
            auth::UseCaseError::UserNotVerified => {
                warn!("User email has not been verified: {e}");
                ApiError::Unauthorized("The user email has not been verified.".to_string())
            }
            auth::UseCaseError::Internal(e) => {
                error!("An internal error occurred: {e}");
                ApiError::Internal("An internal error occurred.".to_string())
            }
            _ => {
                error!("Unexpected domain error: {e}");
                ApiError::Internal("An internal error occurred".to_string())
            }
        }
    }
}