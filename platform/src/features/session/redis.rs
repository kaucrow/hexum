use ::redis::{
    AsyncCommands,
    aio::ConnectionManager,
};
use async_trait::async_trait;
use thiserror::Error;
use anyhow::Result;

use super::*;

#[derive(Clone)]
pub struct RedisAdapter {
    pub conn: ConnectionManager,
}

impl RedisAdapter {
    pub async fn new(conn: ConnectionManager) -> Result<Self> {
        Ok(Self { conn })
    }

    fn format_key(&self, token: &str) -> String {
        format!("session:{token}")
    }
}

#[async_trait]
impl Port for RedisAdapter {
    async fn store_session(
        &self,
        refresh_token: &str,
        payload: SessionPayload,
        ttl_days: u64,
    ) -> Result<(), PortError> {
        let res: Result<_, LocalError> = async {
            let ttl_seconds = ttl_days * 24 * 60 * 60;

            // Saves the key-value pair and sets the expiration
            let key = self.format_key(refresh_token);
            let value = serde_json::to_string(&payload)?;

            let _: () = self.conn.clone().set_ex(key, value, ttl_seconds).await?;

            Ok(())
        }.await;

        res.map_err(Into::into)
    }

    async fn consume_session(
        &self,
        refresh_token: &str
    ) -> Result<Option<SessionPayload>, PortError> {
        let res: Result<_, LocalError> = async {
            // Fetches the user_id and deletes the token
            let key = self.format_key(refresh_token);
            let session_payload: Option<String> = self.conn.clone().get_del(key)
                .await
                .ok();

            if let Some(payload_str) = session_payload {
                let session_payload: SessionPayload = serde_json::from_str(&payload_str)?;
                Ok(Some(session_payload))
            } else {
                Ok(None)
            }
        }.await;

        res.map_err(Into::into)
    }
}

#[derive(Error, Debug)]
pub enum LocalError {
    #[error(transparent)]
    Redis(#[from] ::redis::RedisError),
    #[error(transparent)]
    Uuid(#[from] uuid::Error),
    #[error(transparent)]
    Serialization(#[from] serde_json::Error),
}

impl From<LocalError> for PortError {
    fn from(e: LocalError) -> Self {
        match e {
            LocalError::Redis(e) => PortError::Internal(e.to_string()),
            LocalError::Uuid(e) => PortError::Internal(e.to_string()),
            LocalError::Serialization(e) => PortError::Internal(e.to_string()),
        }
    }
}