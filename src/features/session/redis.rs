use async_trait::async_trait;
use ::redis::AsyncCommands;
use thiserror::Error;
use uuid::Uuid;
use anyhow::Result;

use crate::prelude::*;
use crate::Config;
use super::*;

#[derive(Clone)]
pub struct RedisAdapter {
    pub conn: ::redis::aio::ConnectionManager,
}

impl RedisAdapter {
    pub async fn new(config: &Config) -> Result<Self> {
        let client = ::redis::Client::open(config.redis.url())?;

        let conn = ::redis::aio::ConnectionManager::new(client)
            .await
            .context("Failed to connect to Redis database.")?;

        Ok(Self { conn })
    }
}
impl RedisAdapter {
    async fn do_store_session(&self, refresh_token: &str, user_id: &Uuid, ttl_days: u64) -> Result<(), LocalError> {
        let ttl_seconds = ttl_days * 24 * 60 * 60;

        // Saves the key and sets the expiration
        let key = self.format_key(refresh_token);
        let _: () = self.conn.clone().set_ex(key, user_id.to_string(), ttl_seconds).await?;

        Ok(())
    }

    async fn do_consume_session(&self, refresh_token: &str) -> Result<Option<Uuid>, LocalError> {
        // Fetches the user_id and deletes the token
        let key = self.format_key(refresh_token);
        let user_id: Option<String> = self.conn.clone().get_del(key)
            .await
            .ok();

        if let Some(user_id) = user_id {
            let user_id_uuid = Uuid::try_parse(&user_id)?;
            Ok(Some(user_id_uuid))
        } else {
            Ok(None)
        }
    }

    fn format_key(&self, token: &str) -> String {
        format!("session:{token}")
    }
}

#[async_trait]
impl Port for RedisAdapter {
    async fn store_session(&self, refresh_token: &str, user_id: &Uuid, ttl_days: u64) -> Result<(), PortError> {
        Ok(self.do_store_session(refresh_token, user_id, ttl_days).await?)
    }

    async fn consume_session(&self, refresh_token: &str) -> Result<Option<Uuid>, PortError> {
        Ok(self.do_consume_session(refresh_token).await?)
    }
}

#[derive(Error, Debug)]
pub enum LocalError {
    #[error(transparent)]
    Redis(#[from] ::redis::RedisError),
    #[error(transparent)]
    Uuid(#[from] uuid::Error),
}

impl From<LocalError> for PortError {
    fn from(e: LocalError) -> Self {
        match e {
            LocalError::Redis(e) => PortError::Internal(e.to_string()),
            LocalError::Uuid(e) => PortError::Internal(e.to_string()),
        }
    }
}