use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

use crate::{
    prelude::*,
    features::user::{Role, AuthProvider},
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Port: Send + Sync + 'static {
    /// Stores the refresh token and associates it with a user ID for a given number of days.
    async fn store_session(
        &self,
        refresh_token: &str,
        payload: SessionPayload,
        ttl_days: u64,
    ) -> Result<(), PortError>;

    /// Fetches the user ID associated with the token and deletes the token.
    async fn consume_session(
        &self,
        refresh_token: &str
    ) -> Result<Option<SessionPayload>, PortError>;
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SessionPayload {
    pub user_id: Uuid,
    pub roles: Vec<Role>,
    pub provider: AuthProvider,
}

#[derive(Error, Debug)]
pub enum PortError {
    #[error("Session: {0}")]
    Internal(String)
}