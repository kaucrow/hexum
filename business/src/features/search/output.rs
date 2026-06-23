use crate::{
    prelude::*,
    features::base::pagination,
};
use super::*;

// ─── Internal repository ───────────────────────────────────

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait InternalRepository: Send + Sync + 'static {
    /// Resolve a batch of internal UUIDs to full GameResultItems.
    async fn get_games_by_ids(
        &self,
        ids: &[Uuid],
    ) -> Result<Vec<GameResultItem>, InternalRepositoryError>;

    /// Dynamically builds and executes a query to resolve game UUIDs
    /// from the internal database based on the provided [`GameSearch`] criteria.
    async fn get_game_ids_by_criteria(
        &self,
        search: &GameSearch,
    ) -> Result<Vec<Uuid>, InternalRepositoryError>;

    /// Gets all external platform IDs that match a batch of internal
    /// platform UUIDs.
    async fn get_external_platform_ids_by_internal_ids(
        &self,
        platform_ids: &[Uuid],
    ) -> Result<Vec<u64>, InternalRepositoryError>;
}

#[derive(Error, Debug)]
pub enum InternalRepositoryError {
    #[error("Error in Internal Repository: {0}")]
    Internal(String),
}

// ─── InternalRepository + pagination::InternalRepository ──

pub trait PaginatedInternalRepository: InternalRepository + pagination::InternalRepository {}