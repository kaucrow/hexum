use crate::prelude::*;
use super::*;

/// Result of a popular games query.
pub struct PopularGamesResult {
    pub games: Vec<PopularGameItem>,
    pub total_count: usize,
    /// `true` when results came from internal DB (IGDB was unreachable).
    pub from_internal: bool,
}

#[async_trait]
pub trait UseCase: Send + Sync + 'static {
    // ─── Getters ───
    /// Gets a game from the internal repository.
    async fn get_internal_game(&self, id: &Uuid) -> Result<Option<Game>, UseCaseError>;

    /// Gets a game from the external API.
    async fn get_external_game(&self, id: u64) -> Result<Option<Game>, UseCaseError>;

    /// Gets popular games. Tries IGDB first (sorted by popularity);
    /// falls back to internal DB sorted by view_count if IGDB is unreachable.
    async fn get_popular_games(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<PopularGamesResult, UseCaseError>;
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