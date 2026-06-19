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

    fn format_key(session_id: &str) -> String {
        format!("search:session:{}", session_id)
    }
}

#[async_trait]
impl CacheRepository for RedisAdapter {
    async fn get_search_result_ids(
        &self,
        session_id: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Option<SearchResultPage>, CacheRepositoryError> {
        let res: Result<_, LocalError> = async {
            let key = Self::format_key(session_id);
            let mut conn = self.conn.clone();

            // Check if the list exists by getting its length
            let total_items: usize = conn.llen(&key).await?;

            if total_items == 0 {
                return Ok(None);
            }

            // Check for the EMPTY flag (stored when internal results are 0)
            let first: Option<String> = conn.lindex(&key, 0).await?;
            if first.as_deref() == Some("EMPTY") {
                // Refresh the TTL and return an empty page
                let _: () = conn.expire(&key, CACHE_TTL_SECONDS as i64).await?;
                return Ok(Some(SearchResultPage {
                    ids: vec![],
                    total_items: 0,
                }));
            }

            // Refresh the TTL since the search is active
            let _: () = conn.expire(&key, CACHE_TTL_SECONDS as i64).await?;

            // Let Redis handle the pagination via LRANGE
            let start = offset as isize;
            let end = (offset + limit - 1) as isize;

            let page_strs: Vec<String> = conn.lrange(&key, start, end).await?;
            let page_ids: Vec<Uuid> = page_strs
                .iter()
                .filter_map(|s| Uuid::parse_str(s).ok())
                .collect();

            Ok(Some(SearchResultPage {
                ids: page_ids,
                total_items,
            }))
        }
        .await;

        res.map_err(Into::into)
    }

    async fn get_all_search_result_ids(
        &self,
        session_id: &str,
    ) -> Result<Option<Vec<Uuid>>, CacheRepositoryError> {
        let res: Result<_, LocalError> = async {
            let key = Self::format_key(session_id);
            let mut conn = self.conn.clone();

            let total_items: usize = conn.llen(&key).await?;

            if total_items == 0 {
                return Ok(None);
            }

            // Check for the EMPTY flag (stored when internal results are 0)
            let first: Option<String> = conn.lindex(&key, 0).await?;
            if first.as_deref() == Some("EMPTY") {
                let _: () = conn.expire(&key, CACHE_TTL_SECONDS as i64).await?;
                return Ok(Some(vec![]));
            }

            let _: () = conn.expire(&key, CACHE_TTL_SECONDS as i64).await?;

            // LRANGE 0 -1 will get everything from start to finish
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

    async fn cache_search_result_ids(
        &self,
        session_id: &str,
        ids: &[Uuid],
    ) -> Result<(), CacheRepositoryError> {
        let res: Result<_, LocalError> = async {
            let key = Self::format_key(session_id);
            let mut conn = self.conn.clone();

            // Use a Redis transaction to ensure atomicity
            let mut pipe = ::redis::pipe();
            pipe.atomic();

            if ids.is_empty() {
                // Push a dummy string
                pipe.rpush(&key, "EMPTY").ignore();
            } else {
                // Convert UUIDs to their string representation for storage
                let str_ids: Vec<String> = ids.iter().map(|id| id.to_string()).collect();

                pipe.rpush(&key, &str_ids).ignore();    // Push all ID strings into the list
            }

            // Set the TTL
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