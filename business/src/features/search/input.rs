use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

use crate::prelude::*;
use super::{
    SearchSession,
    InternalRepositoryError,
    ExternalRepositoryError,
    CacheRepositoryError,
};

#[async_trait]
pub trait UseCase: Send + Sync + 'static {
    // ─── Getters ───
    async fn search_for_game(
        &self,
        query: Option<&str>,
        search_id: Option<Uuid>,
        limit: usize,
        offset: usize,
    ) -> Result<SearchSession, UseCaseError>;
}

#[derive(Error, Debug)]
pub enum UseCaseError {
    /// Videogame API error.
    #[error("External error in Videogame API Search: {0}")]
    VideogameApi(String),

    /// Unexpected internal error.
    #[error("Search service: {0}.")]
    Internal(String),
}

impl From<InternalRepositoryError> for UseCaseError {
    fn from(e: InternalRepositoryError) -> Self {
        UseCaseError::Internal(e.to_string())
    }
}

impl From<CacheRepositoryError> for UseCaseError {
    fn from(e: CacheRepositoryError) -> Self {
        UseCaseError::Internal(e.to_string())
    }
}

impl From<ExternalRepositoryError> for UseCaseError {
    fn from(e: ExternalRepositoryError) -> Self {
        UseCaseError::VideogameApi(e.to_string())
    }
}