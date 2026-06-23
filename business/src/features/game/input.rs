use crate::prelude::*;
use super::*;

#[async_trait]
pub trait UseCase: Send + Sync + 'static {
    // ─── Getters ───
    /// Gets a game from the internal repository.
    async fn get_internal_game(&self, id: &Uuid) -> Result<Option<Game>, UseCaseError>;

    /// Gets a game from the external API.
    async fn get_external_game(&self, id: u64) -> Result<Option<Game>, UseCaseError>;
}

#[derive(Error, Debug)]
pub enum UseCaseError {
    /// Videogame API error.
    #[error("External error in Videogame API (Game Service): {0}")]
    VideogameApi(String),

    /// Unexpected internal error.
    #[error("Game service: {0}.")]
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