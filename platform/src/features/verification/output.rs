use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Port: Send + Sync + 'static {
    // Stores a token mapped to a User ID with an expiry
    async fn store_verification_token(&self, user_id: &Uuid, token: &str, expires_in_secs: u64) -> Result<(), PortError>;
    // Retrieves user_id from token
    async fn consume_verification_token(&self, token: &str) -> Result<Uuid, PortError>;
}

#[derive(Error, Debug)]
pub enum PortError {
    #[error("The verification token is invalid or expired.")]
    VerificationTokenInvalid,

    #[error("The verification code is already in use. Please try again.")]
    CodeInUse,

    #[error("Verification: {0}")]
    Internal(String),
}
