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

    /// Increments the view_count for a game by 1.
    async fn increment_view_count(&self, id: &Uuid) -> Result<(), InternalRepositoryError>;

    /// Gets preview popular game items sorted by view_count descending.
    async fn get_popular_items_by_view_count(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<PopularGameItem>, InternalRepositoryError>;

    /// Returns the total number of games in the internal database.
    async fn count_all_games(&self) -> Result<usize, InternalRepositoryError>;
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

    /// Fetches popular games from the external API sorted by popularity.
    /// Returns the games and the total count.
    async fn get_popular_games(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<(Vec<PopularGameItem>, usize), ExternalRepositoryError>;
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