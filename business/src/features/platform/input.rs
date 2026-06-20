use async_trait::async_trait;
use thiserror::Error;

use super::*;

#[async_trait]
pub trait UseCase: Send + Sync + 'static {
    // ─── Getters ───
    async fn get_platforms(&self) -> Result<Vec<Platform>, UseCaseError>;

    // ─── Commands ───
    async fn sync_db_platforms(&self) -> Result<(), UseCaseError>;
}

#[derive(Error, Debug)]
pub enum UseCaseError {
    /// Videogame API error.
    #[error("External error in Videogame API Platform: {0}")]
    VideogameApi(String),

    /// Unexpected internal error.
    #[error("Platform service: {0}.")]
    Internal(String),
}

impl From<InternalRepositoryError> for UseCaseError {
    fn from(e: InternalRepositoryError) -> Self {
        match e {
            InternalRepositoryError::Internal(msg) => UseCaseError::Internal(msg),
        }
    }
}

impl From<ExternalRepositoryError> for UseCaseError {
    fn from(e: ExternalRepositoryError) -> Self {
        match e {
            ExternalRepositoryError::VideogameApi(msg) => UseCaseError::VideogameApi(msg),
            ExternalRepositoryError::Internal(msg) => UseCaseError::Internal(msg),
        }
    }
}