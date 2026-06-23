use crate::prelude::*;
use super::*;

// ─── Internal repository ──────────────────────────────────────

#[async_trait]
pub trait InternalRepository: Send + Sync + 'static {
    // ─── Getters ───
    /// Gets a game's entire data.
    async fn get_game(&self, id: &Uuid) -> Result<Option<Game>, InternalRepositoryError>;

    /// Inserts a game from the external API into the local repo, or updates its
    /// data if it already exists in the local repo. Returns the internal game ID.
    async fn sync_db_game_from_external(&self, game: &Game) -> Result<Uuid, InternalRepositoryError>;
}

#[derive(Error, Debug)]
pub enum InternalRepositoryError {
    /// Unexpected internal error.
    #[error("Error in Internal Repository: {0}.")]
    Internal(String),
}

// ─── External repository ──────────────────────────────────────

#[async_trait]
pub trait ExternalRepository: Send + Sync + 'static {
    // ─── Getters ───
    /// Gets a game's entire data.
    async fn get_game(&self, id: u64) -> Result<Option<Game>, ExternalRepositoryError>;
}

#[derive(Error, Debug)]
pub enum ExternalRepositoryError {
    /// Videogame API error.
    #[error("External error in Videogame API (Game): {0}")]
    VideogameApi(String),

    /// Unexpected internal error.
    #[error("Error in External Repository: {0}.")]
    Internal(String),
}