use uuid::Uuid;
use async_trait::async_trait;
use thiserror::Error;

use crate::prelude::*;
use super::*;

// ────────────────────────────────────────────────
//  Local Repository
// ────────────────────────────────────────────────
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait LocalRepository: Send + Sync + 'static {
    // ─── Getters ───
    async fn get_recipe_search_ids(&self, query: &str) -> Result<Vec<Uuid>, LocalRepositoryError>;
    async fn get_recipe_search_data_by_ids(&self, ids: &Vec<Uuid>) -> Result<Vec<RecipeSearchResult>, LocalRepositoryError>;

    async fn get_recipe_by_id(&self, id: &Uuid) -> Result<Option<Recipe>, LocalRepositoryError>;

    async fn get_tag_search_matches(&self, query: &str, limit: usize) -> Result<Vec<String>, LocalRepositoryError>;
}

#[derive(Error, Debug)]
pub enum LocalRepositoryError {
    #[error("Recipe local repository: {0}")]
    Internal(String),
}

// ────────────────────────────────────────────────
//  Cache Repository
// ────────────────────────────────────────────────
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait CacheRepository: Send + Sync + 'static {
    // ─── Search results caching ───
    async fn get_recipe_ids(&self, key: &str) -> Result<Option<Vec<Uuid>>, CacheRepositoryError>;
    async fn set_recipe_ids(&self, key: &str, ids: &[Uuid], ttl_secs: u64) -> Result<(), CacheRepositoryError>;

    // ─── Individual full recipes caching ───
    async fn get_recipe(&self, id: &Uuid) -> Result<Option<Recipe>, CacheRepositoryError>;
    async fn set_recipe(&self, id: &Uuid, data: &Recipe, ttl_secs: u64) -> Result<(), CacheRepositoryError>;
}

#[derive(Error, Debug)]
pub enum CacheRepositoryError {
    #[error("Recipe cache repository: {0}")]
    Internal(String),
}