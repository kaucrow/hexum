use async_trait::async_trait;
use ::redis::{
    AsyncCommands,
    aio::ConnectionManager,
};

use super::*;

pub struct RedisCacheAdapter {
    conn: ConnectionManager,
}

impl RedisCacheAdapter {
    pub fn new(conn: ConnectionManager) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl CacheRepository for RedisCacheAdapter {
    // ───────────────────────────────────────────────────
    //  Search Results Caching
    // ───────────────────────────────────────────────────

    async fn get_search_results(&self, key: &str) -> Result<Option<Vec<domain::RecipeSearchResult>>, CacheRepositoryError> {
        let mut conn = self.conn.clone();

        // Fetch the raw payload string from redis
        let raw_json: Option<String> = conn.get(key).await?;

        // Deserialize the string if a cache hit occurred
        match raw_json {
            Some(json_str) => {
                let candidates = serde_json::from_str(&json_str)?;
                Ok(Some(candidates))
            }
            None => Ok(None), // Pure cache miss
        }
    }

    async fn set_search_results(&self, key: &str, search_results: &[domain::RecipeSearchResult], ttl_secs: u64) -> Result<(), CacheRepositoryError> {
        let mut conn = self.conn.clone();

        // Serialize the candidates
        let json_str = serde_json::to_string(search_results)?;

        // Set in redis
        let _: () = conn.set_ex(key, json_str, ttl_secs).await?;

        Ok(())
    }

    // ───────────────────────────────────────────────────
    //  Individual Full Recipes Caching
    // ───────────────────────────────────────────────────

    async fn get_recipe(&self, id: &str) -> Result<Option<Recipe>, CacheRepositoryError> {
        let mut conn = self.conn.clone();

        let raw_json: Option<String> = conn.get(id).await?;

        match raw_json {
            Some(json_str) => {
                let recipe = serde_json::from_str(&json_str)?;
                Ok(Some(recipe))
            }
            None => Ok(None),
        }
    }

    async fn set_recipe(&self, id: &str, recipe: &Recipe, ttl_secs: u64) -> Result<(), CacheRepositoryError> {
        let mut conn = self.conn.clone();

        let json_str = serde_json::to_string(recipe)?;

        let _: () = conn.set_ex(id, json_str, ttl_secs).await?;

        Ok(())
    }
}

impl From<::redis::RedisError> for CacheRepositoryError {
    fn from(e: ::redis::RedisError) -> Self {
        CacheRepositoryError::Internal(e.to_string())
    }
}

impl From<serde_json::Error> for CacheRepositoryError {
    fn from(e: serde_json::Error) -> Self {
        CacheRepositoryError::Internal(e.to_string())
    }
}