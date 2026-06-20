use async_trait::async_trait;
use ::redis::{AsyncCommands, aio::ConnectionManager};
use uuid::Uuid;
use thiserror::Error;

use super::*;

const CACHE_TTL_SECONDS: u64 = 600; // 10 minutes

#[derive(Clone)]
pub struct RedisAdapter {
    conn: ConnectionManager,
}

impl RedisAdapter {
    pub fn new(conn: ConnectionManager) -> Self {
        Self { conn }
    }

    fn format_key(&self, prefix_key: &str, session_id: &str) -> String {
        format!("{}:{}", prefix_key, session_id)
    }
}

#[async_trait]
impl CacheRepository for RedisAdapter {
    async fn get_cached_ids(
        &self,
        prefix_key: &str,
        session_id: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Option<CachedIdsPage>, CacheRepositoryError> {
        let res: Result<_, LocalError> = async {
            let key = self.format_key(prefix_key, session_id);
            let mut conn = self.conn.clone();

            let total_items: usize = conn.llen(&key).await?;

            if total_items == 0 {
                return Ok(None);
            }

            // Check for the 'EMPTY' flag
            let first: Option<String> = conn.lindex(&key, 0).await?;
            if first.as_deref() == Some("EMPTY") {
                let _: () = conn.expire(&key, CACHE_TTL_SECONDS as i64).await?;
                return Ok(Some(CachedIdsPage { ids: vec![], total_items: 0 }));
            }

            let _: () = conn.expire(&key, CACHE_TTL_SECONDS as i64).await?;

            let start = offset as isize;
            let end = (offset + limit - 1) as isize;

            let page_strs: Vec<String> = conn.lrange(&key, start, end).await?;
            let page_ids: Vec<Uuid> = page_strs
                .iter()
                .filter_map(|s| Uuid::parse_str(s).ok())
                .collect();

            Ok(Some(CachedIdsPage { ids: page_ids, total_items }))
        }
        .await;

        res.map_err(Into::into)
    }

    async fn get_all_cached_ids(
        &self,
        prefix_key: &str,
        session_id: &str,
    ) -> Result<Option<Vec<Uuid>>, CacheRepositoryError> {
        let res: Result<_, LocalError> = async {
            let key = self.format_key(prefix_key, session_id);
            let mut conn = self.conn.clone();

            let total_items: usize = conn.llen(&key).await?;

            if total_items == 0 {
                return Ok(None);
            }

            // Check for the 'EMPTY' flag
            let first: Option<String> = conn.lindex(&key, 0).await?;
            if first.as_deref() == Some("EMPTY") {
                let _: () = conn.expire(&key, CACHE_TTL_SECONDS as i64).await?;
                return Ok(Some(vec![]));
            }

            let _: () = conn.expire(&key, CACHE_TTL_SECONDS as i64).await?;

            let all_strs: Vec<String> = conn.lrange(&key, 0, -1).await?;
            let all_ids: Vec<Uuid> = all_strs
                .iter()
                .filter_map(|s| Uuid::parse_str(s).ok())
                .collect();

            Ok(Some(all_ids))
        }
        .await;

        res.map_err(Into::into)
    }

    async fn cache_ids(
        &self,
        prefix_key: &str,
        session_id: &str,
        ids: &[Uuid],
    ) -> Result<(), CacheRepositoryError> {
        let res: Result<_, LocalError> = async {
            let key = self.format_key(prefix_key, session_id);
            let mut conn = self.conn.clone();

            let mut pipe = ::redis::pipe();
            pipe.atomic();

            if ids.is_empty() {
                pipe.rpush(&key, "EMPTY").ignore();
            } else {
                let str_ids: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
                pipe.rpush(&key, &str_ids).ignore();
            }

            pipe.expire(&key, CACHE_TTL_SECONDS as i64).ignore();

            let _: () = pipe.query_async(&mut conn).await?;

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
}

impl From<LocalError> for CacheRepositoryError {
    fn from(e: LocalError) -> Self {
        match e {
            LocalError::Redis(e) => CacheRepositoryError::Internal(e.to_string()),
        }
    }
}