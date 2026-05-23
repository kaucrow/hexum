use async_trait::async_trait;
use thiserror::Error;

#[async_trait]
pub trait Port: Send + Sync {
    /// Exchanges a code for Google user info
    async fn get_google_user_info_by_code(&self, code: &str) -> Result<GoogleUserInfo, PortError>;
    /// Exchanges a code for GitHub user info
    async fn get_github_user_info_by_code(&self, code: &str) -> Result<GitHubUserInfo, PortError>;
}

pub struct GoogleUserInfo {
    pub email: String,
    pub external_id: String,
}

pub struct GitHubUserInfo {
    pub email: String,
    pub external_id: i64,
    pub username: String,
}

#[derive(Debug, Error)]
pub enum PortError {
    #[error("The authorization code provided is invalid or has expired")]
    InvalidCode,

    #[error("A network error occurred while communicating with the OAuth provider: {0}")]
    NetworkError(String),

    #[error("Failed to parse user info response from the OAuth provider")]
    ParseError,

    #[error("OAuth: {0}")]
    Internal(String),
}