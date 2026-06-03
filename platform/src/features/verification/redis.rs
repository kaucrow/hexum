use ::redis::{
    AsyncCommands,
    aio::ConnectionManager,
};
use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;
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

    async fn do_store_verification_token(&self, user_id: &Uuid, token: &str, expires_in_secs: u64) -> Result<(), LocalError> {
        let key = self.format_key(token);

        // Use SET NX to ensure the code is unique — only set if the key doesn't already exist
        let set: Option<String> = self.conn.clone().set_nx(key.clone(), user_id.to_string()).await?;

        match set {
            Some(_) => {
                // Key was set successfully, now set the expiry
                let _: () = self.conn.clone().expire(key, expires_in_secs as i64).await?;
                Ok(())
            }
            None => {
                // Key already exists — code collision
                Err(LocalError::CodeInUse)
            }
        }
    }

    async fn do_consume_verification_token(&self, token: &str) -> Result<Uuid, LocalError> {
        let key = self.format_key(token);

        let user_id: String = self.conn.clone().get_del::<&str, Option<String>>(&key)
            .await?
            .ok_or(LocalError::VerificationTokenInvalid)?;

        let user_id_uuid = Uuid::try_parse(&user_id)?;

        Ok(user_id_uuid)
    }

    fn format_key(&self, token: &str) -> String {
        format!("verify:{token}")
    }
}

#[async_trait]
impl Port for RedisAdapter {
    async fn store_verification_token(&self, user_id: &Uuid, token: &str, expires_in_secs: u64) -> Result<(), PortError> {
        Ok(self.do_store_verification_token(user_id, token, expires_in_secs).await?)
    }

    async fn consume_verification_token(&self, token: &str) -> Result<Uuid, PortError> {
        Ok(self.do_consume_verification_token(token).await?)
    }
}

#[derive(Error, Debug)]
pub enum LocalError {
    #[error("")]
    VerificationTokenInvalid,
    #[error("The verification code is already in use.")]
    CodeInUse,
    #[error(transparent)]
    Redis(#[from] ::redis::RedisError),
    #[error(transparent)]
    Uuid(#[from] uuid::Error),
}

impl From<LocalError> for PortError {
    fn from(e: LocalError) -> Self {
        match e {
            LocalError::VerificationTokenInvalid => PortError::VerificationTokenInvalid,
            LocalError::CodeInUse => PortError::CodeInUse,
            LocalError::Redis(e) => PortError::Internal(e.to_string()),
            LocalError::Uuid(e) => PortError::Internal(e.to_string()),
        }
    }
}