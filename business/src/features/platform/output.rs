use async_trait::async_trait;
use thiserror::Error;

use super::*;

// ─── Internal repository ──────────────────────────────────────

#[async_trait]
pub trait InternalRepository: Send + Sync + 'static {
    async fn upsert_platforms(&self, platforms: &[Platform]) -> Result<(), InternalRepositoryError>;

    async fn get_platforms(&self) -> Result<Vec<Platform>, InternalRepositoryError>;
}

#[derive(Error, Debug)]
pub enum InternalRepositoryError {
    /// Unexpected internal error.
    #[error("Search service: {0}.")]
    Internal(String),
}

// ─── External repository ──────────────────────────────────────

#[async_trait]
pub trait ExternalRepository: Send + Sync + 'static {
    async fn get_platforms(&self) -> Result<Vec<Platform>, ExternalRepositoryError>;
}

#[derive(Error, Debug)]
pub enum ExternalRepositoryError {
    /// Videogame API error.
    #[error("External error in Videogame API Search: {0}")]
    VideogameApi(String),

    /// Unexpected internal error.
    #[error("Search service: {0}.")]
    Internal(String),
}