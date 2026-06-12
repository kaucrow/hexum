use ::redis::{
    AsyncCommands,
    aio::ConnectionManager,
};
use async_trait::async_trait;
use thiserror::Error;

use super::*;

#[derive(Clone)]
pub struct RedisAdapter {
    pub conn: ConnectionManager,
}

impl RedisAdapter {
    pub async fn new(conn: ConnectionManager) -> Self {
        Self { conn }
    }

    fn format_key(&self, token: &str) -> String {
        format!("verify:{token}")
    }
}

#[async_trait]
impl Port for RedisAdapter {
    async fn store_verification_token(&self, payload: &str, token: &str, expires_in_secs: u64) -> Result<(), PortError> {
        let res: Result<_, LocalError> = async {
            let key = self.format_key(token);

            // Only sets the code if the key doesn't already exist
            let set: Option<String> = self.conn.clone().set_nx(&key, payload).await?;

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
        }.await;

        res.map_err(Into::into)
    }

    async fn consume_verification_token(&self, token: &str) -> Result<String, PortError> {
        let res: Result<_, LocalError> = async {
            let key = self.format_key(token);

            let payload: String = self.conn.clone().get_del::<&str, Option<String>>(&key)
                .await?
                .ok_or(LocalError::VerificationTokenInvalid)?;

            Ok(payload)
        }.await;

        res.map_err(Into::into)
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