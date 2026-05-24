use async_trait::async_trait;
use thiserror::Error;

use crate::features::user;

#[async_trait]
pub trait UseCase: Send + Sync + 'static {
    /// Logs in a user via username/email.
    async fn login_user(&self, identity: &str, passwd: &str) -> Result<AuthTokens, UseCaseError>;
    async fn login_user_via_google(&self, code: &str) -> Result<AuthTokens, UseCaseError>;
    async fn login_user_via_github(&self, code: &str) -> Result<AuthTokens, UseCaseError>;
    async fn verify_user(&self, access_token: &str) -> Result<user::User, UseCaseError>;
    async fn refresh_session(&self, refresh_token: &str) -> Result<AuthTokens, UseCaseError>;
    async fn logout_user(&self, refresh_token: &str) -> Result<(), UseCaseError>;
}

pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Error, Debug)]
pub enum UseCaseError {
    #[error("The password provided is invalid.")]
    InvalidPassword,

    #[error("The access token provided is invalid: {0}")]
    InvalidAccessToken(String),

    #[error("The refresh token provided is invalid")]
    InvalidRefreshToken,

    #[error("{0}")]
    InvalidOAuthCode(String),

    #[error("The user could not be found.")]
    UserNotFound,

    #[error("The user is inactive.")]
    UserInactive,

    #[error("The user email hasn't been verified.")]
    UserNotVerified,

    #[error("Parsing error: {0}")]
    Parse(String),

    #[error("Auth service: {0}")]
    Internal(String),
}