use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

#[async_trait]
pub trait Port: Send + Sync + 'static {
    // Stores the refresh token and associates it with a user ID for a given number of days
    async fn store_session(&self, refresh_token: &str, user_id: &Uuid, ttl_days: u64) -> Result<(), PortError>;
    // Fetches the user ID associated with the token and deletes the token
    async fn consume_session(&self, refresh_token: &str) -> Result<Option<Uuid>, PortError>;
}

#[derive(Error, Debug)]
pub enum PortError {
    #[error("Session: {0}")]
    Internal(String)
}