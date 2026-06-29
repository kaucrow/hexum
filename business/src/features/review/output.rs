use async_trait::async_trait;
use thiserror::Error;

use crate::prelude::*;
use super::Review;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Repository: Send + Sync + 'static {
    /// Insert or update a review.
    async fn upsert_review(&self, review: &Review) -> Result<Review, RepositoryError>;

    /// Get an existing review by user and game.
    async fn get_review_by_user_and_game(
        &self,
        user_id: &Uuid,
        game_id: &Uuid,
    ) -> Result<Option<Review>, RepositoryError>;

    /// Recalculate the game's rating, critic_rating, and total_rating_count
    /// from all reviews for that game.
    async fn recalculate_game_ratings(&self, game_id: &Uuid) -> Result<(), RepositoryError>;
}

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Review repository: {0}")]
    Internal(String),
}