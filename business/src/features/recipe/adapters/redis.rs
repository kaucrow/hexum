use uuid::Uuid;
use async_trait::async_trait;
use chrono::{Utc, Datelike, Duration};
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

    async fn get_recipe_ids(&self, key: &str) -> Result<Option<Vec<Uuid>>, CacheRepositoryError> {
        let mut conn = self.conn.clone();

        // Fetch the raw payload string
        let raw_json: Option<String> = conn.get(key).await?;

        // Deserialize the string if a cache hit occurred
        match raw_json {
            Some(json_str) => {
                let candidates = serde_json::from_str(&json_str)?;
                Ok(Some(candidates))
            }
            None => Ok(None),   // Pure cache miss
        }
    }

    async fn set_recipe_ids(&self, key: &str, ids: &[Uuid], ttl_secs: u64) -> Result<(), CacheRepositoryError> {
        let mut conn = self.conn.clone();

        // Serialize the IDs
        let json_str = serde_json::to_string(ids)?;

        // Set in redis
        let _: () = conn.set_ex(key, json_str, ttl_secs).await?;

        Ok(())
    }

    // ───────────────────────────────────────────────────
    //  Individual Full Recipes Caching
    // ───────────────────────────────────────────────────

    async fn get_recipe(&self, id: &Uuid) -> Result<Option<Recipe>, CacheRepositoryError> {
        let mut conn = self.conn.clone();
        let key = format!("recipe:{}", id.to_string());

        let raw_json: Option<String> = conn.get(&key).await?;

        match raw_json {
            Some(json_str) => {
                // Reset the TTL
                let _: () = conn.expire(key, 1800).await?;

                let recipe = serde_json::from_str(&json_str)?;
                Ok(Some(recipe))
            }
            None => Ok(None),
        }
    }

    async fn set_recipe(&self, id: &Uuid, recipe: &Recipe, ttl_secs: u64) -> Result<(), CacheRepositoryError> {
        let mut conn = self.conn.clone();
        let key = format!("recipe:{}", id.to_string());

        let json_str = serde_json::to_string(recipe)?;

        let _: () = conn.set_ex(key, json_str, ttl_secs).await?;

        Ok(())
    }

    async fn get_yesterday_most_viewed_recipe_ids(&self, limit: usize) -> Result<Option<Vec<Uuid>>, CacheRepositoryError> {
        let mut conn = self.conn.clone();
        let yesterday = Utc::now() - Duration::days(1);
        let key = format!(
            "recipe:views:{:02}-{:02}-{}",
            yesterday.day(), yesterday.month(), yesterday.year()
        );

        // Get redis data as a Vec of (Field, Value)
        let redis_data: Vec<(String, String)> = conn.hgetall(&key).await?;

        // If the hash doesn't exist or has no entries, return None
        if redis_data.is_empty() {
            return Ok(None);
        }

        // Parse strings into concrete types: (Recipe UUID, View count i32)
        let mut parsed_views: Vec<(Uuid, i32)> = redis_data
            .into_iter()
            .filter_map(|(id_str, count_str)| {
                let id = Uuid::parse_str(&id_str).ok()?;
                let count = count_str.parse::<i32>().ok()?;
                Some((id, count))
            })
            .collect();

        // Sort the vector by view counts (highest views first)
        parsed_views.sort_by(|a, b| b.1.cmp(&a.1));

        // Extract just the UUIDs
        let top_recipe_ids: Vec<Uuid> = parsed_views
            .into_iter()
            .take(limit)
            .map(|(id, _count)| id)
            .collect();

        Ok(Some(top_recipe_ids))
    }

    async fn track_recipe_views(&self, recipe_id: &Uuid) -> Result<(), CacheRepositoryError> {
        let mut conn = self.conn.clone();
        let now = Utc::now();
        let key = format!(
            "recipe:views:{:02}-{:02}-{}",
            now.day(), now.month(), now.year()
        );

        // Use a pipeline to increment the counter and reset the expiration safety window
        let _: ((), ()) = ::redis::pipe()
            .hincr(&key, recipe_id.to_string(), 1)
            .expire(&key, 100000)   // 27 hours
            .query_async(&mut conn)
            .await?;

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