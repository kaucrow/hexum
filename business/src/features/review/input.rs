use crate::prelude::*;
use platform::features::user;
use super::Review;

#[async_trait]
pub trait UseCase: Send + Sync + 'static {
    // ─── Getters ───
    /// Gets all the user & critic reviews for a specific game.
    async fn get_game_reviews(&self, game_id: &Uuid) -> Result<Vec<Review>, UseCaseError>;

    // ─── Commands ───
    /// Submit or update a review for a game. The review type (user/critic)
    /// is determined automatically from the user's roles.
    async fn submit_review(
        &self,
        user_id: &Uuid,
        roles: &[user::Role],
        game_id: &Uuid,
        rating: i32,
        title: &str,
        content: &str,
    ) -> Result<Review, UseCaseError>;
}

#[derive(Error, Debug)]
pub enum UseCaseError {
    #[error("Game not found.")]
    GameNotFound,

    #[error("Rating must be between 0 and 100.")]
    InvalidRating,

    #[error("Review service: {0}")]
    Internal(String),
}