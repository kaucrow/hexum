use crate::prelude::*;
use super::Review;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait InternalRepository: Send + Sync + 'static {
    // ─── Getters ───
    /// Gets all the user & critic reviews for a specific game.
    async fn get_game_reviews(&self, game_id: &Uuid) -> Result<Vec<Review>, InternalRepositoryError>;

    /// Get an existing review by user and game.
    async fn get_review_by_user_and_game(
        &self,
        user_id: &Uuid,
        game_id: &Uuid,
    ) -> Result<Option<Review>, InternalRepositoryError>;

    // ─── Commands ───
    /// Insert or update a review.
    async fn upsert_review(&self, review: &Review) -> Result<Review, InternalRepositoryError>;

    /// Recalculate the game's rating, critic_rating, and total_rating_count
    /// from all reviews for that game.
    async fn recalculate_game_ratings(&self, game_id: &Uuid) -> Result<(), InternalRepositoryError>;
}

#[derive(Error, Debug)]
pub enum InternalRepositoryError {
    #[error("Review internal repository: {0}")]
    Internal(String),
}