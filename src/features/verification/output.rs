use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

#[async_trait]
pub trait Port: Send + Sync + 'static {
    // Stores a token mapped to a User ID with an expiry
    async fn store_verification_token(&self, user_id: &Uuid, token: &str, expires_in_secs: u64) -> Result<(), PortError>;
    // Retrieves user_id from token
    async fn consume_verification_token(&self, token: &str) -> Result<Uuid, PortError>;
}

#[derive(Error, Debug)]
pub enum PortError {
    #[error("{0}")]
    VerificationTokenInvalid(String),

    #[error("Verification: {0}")]
    Internal(String),
}