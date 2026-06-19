use async_trait::async_trait;
use ::redis::{AsyncCommands, aio::ConnectionManager};
use thiserror::Error;

use super::*;

/// TTL for cached search exclusion IDs, in seconds.
const CACHE_TTL_SECONDS: u64 = 600; // 10 minutes

#[derive(Clone)]
pub struct RedisAdapter {
    conn: ConnectionManager,
}

impl RedisAdapter {
    pub fn new(conn: ConnectionManager) -> Self {
        Self { conn }
    }

    fn format_key(query: &str) -> String {
        format!("search:exclusion:{}", query)
    }
}

#[async_trait]
impl CacheRepository for RedisAdapter {
    async fn get_search_exclusion_ids(
        &self,
        query: &str,
    ) -> Result<Option<Vec<u64>>, CacheRepositoryError> {
        let res: Result<_, LocalError> = async {
            let key = Self::format_key(query);
            let raw: Option<String> = self.conn.clone().get(&key).await?;

            match raw {
                Some(json) => {
                    let ids: Vec<u64> = serde_json::from_str(&json)?;
                    Ok(Some(ids))
                }
                None => Ok(None),
            }
        }
        .await;

        res.map_err(Into::into)
    }

    async fn cache_search_exclusion_ids(
        &self,
        query: &str,
        ids: &[u64],
    ) -> Result<(), CacheRepositoryError> {
        let res: Result<_, LocalError> = async {
            let key = Self::format_key(query);
            let json = serde_json::to_string(ids)?;
            let _: () = self.conn.clone().set_ex(&key, json, CACHE_TTL_SECONDS).await?;
            Ok(())
        }
        .await;

        res.map_err(Into::into)
    }
}

#[derive(Error, Debug)]
pub enum LocalError {
    #[error(transparent)]
    Redis(#[from] ::redis::RedisError),

    #[error(transparent)]
    Serialization(#[from] serde_json::Error),
}

impl From<LocalError> for CacheRepositoryError {
    fn from(e: LocalError) -> Self {
        match e {
            LocalError::Redis(e) => CacheRepositoryError::Internal(e.to_string()),
            LocalError::Serialization(e) => CacheRepositoryError::Internal(e.to_string()),
        }
    }
}