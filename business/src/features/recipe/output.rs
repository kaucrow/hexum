use uuid::Uuid;
use async_trait::async_trait;
use thiserror::Error;

use crate::prelude::*;
use super::{Recipe, RecipeSearchResult, RecipeOrigin};

// ────────────────────────────────────────────────
//  Local Repository
// ────────────────────────────────────────────────
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait LocalRepository: Send + Sync + 'static {
    // ─── Getters ───
    async fn get_recipe_search_results(&self, name: &str) -> Result<Vec<RecipeSearchResult>, LocalRepositoryError>;
}

#[derive(Error, Debug)]
pub enum LocalRepositoryError {
    #[error(transparent)]
    Conflict(#[from] ConflictError),
    #[error("User repository: {0}")]
    Internal(String),
}

// ────────────────────────────────────────────────
//  External Repository
// ────────────────────────────────────────────────
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ExternalRepository: Send + Sync + 'static {
    async fn search_by_name(&self, name: &str) -> Result<Vec<RecipeSearchResult>, ExternalRepositoryError>;
}

#[derive(Error, Debug)]
pub enum ExternalRepositoryError {
    #[error(transparent)]
    Conflict(#[from] ConflictError),
    #[error("Recipe external repository: {0}")]
    Network(String),
    #[error("Recipe external repository: {0}")]
    Serialization(String),
    #[error("Recipe external repository: {0}")]
    Internal(String),
}

// ────────────────────────────────────────────────
//  Cache Repository
// ────────────────────────────────────────────────
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait CacheRepository: Send + Sync + 'static {
    // ─── Search results caching ───
    async fn get_search_results(&self, key: &str) -> Result<Option<Vec<RecipeSearchResult>>, CacheRepositoryError>;
    async fn set_search_results(&self, key: &str, candidates: &[RecipeSearchResult], ttl_secs: u64) -> Result<(), CacheRepositoryError>;

    // ─── Individual full recipes caching ───
    async fn get_recipe(&self, id: &str) -> Result<Option<Recipe>, CacheRepositoryError>;
    async fn set_recipe(&self, id: &str, data: &Recipe, ttl_secs: u64) -> Result<(), CacheRepositoryError>;
}

#[derive(Error, Debug)]
pub enum CacheRepositoryError {
    #[error(transparent)]
    Conflict(#[from] ConflictError),
    #[error("User repository: {0}")]
    Internal(String),
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ConflictError {
    #[error("The username provided is already in use.")]
    UsernameInUse,
    #[error("The email provided is already in use.")]
    EmailInUse,
}