use platform::features::user;

use crate::prelude::*;
use super::*;

#[derive(Clone)]
pub struct Service {
    internal_repo: Arc<dyn InternalRepository>,
}

impl Service {
    pub fn new(internal_repo: Arc<dyn InternalRepository>) -> Self {
        Self { internal_repo }
    }
}

#[async_trait]
impl UseCase for Service {
    /// Gets the user & critic reviews for a game
    async fn get_game_reviews(&self, game_id: &Uuid) -> Result<Vec<Review>, UseCaseError> {
        Ok(self.internal_repo.get_game_reviews(game_id).await?)
    }

    async fn submit_review(
        &self,
        user_id: &Uuid,
        roles: &[user::Role],
        game_id: &Uuid,
        rating: i32,
        title: &str,
        content: &str,
    ) -> Result<Review, UseCaseError> {
        // Validate rating range
        if !(0..=100).contains(&rating) {
            return Err(UseCaseError::InvalidRating);
        }

        // Determine review type from roles
        let review_type = if roles.contains(&user::Role::Critic) {
            ReviewType::Critic
        } else {
            ReviewType::User
        };

        // Check if user already has a review for this game (for the ID reuse)
        let existing = self.internal_repo
            .get_review_by_user_and_game(user_id, game_id)
            .await?;

        let review_id = existing
            .as_ref()
            .map(|r| r.id)
            .unwrap_or_else(Uuid::new_v4);

        let review = Review {
            id: review_id,
            user_id: *user_id,
            game_id: *game_id,
            review_type,
            rating,
            title: title.to_string(),
            content: content.to_string(),
            created_at: existing.as_ref().map(|r| r.created_at).unwrap_or_default(),
            updated_at: Utc::now(),
        };

        let saved = self.internal_repo
            .upsert_review(&review)
            .await?;

        // Recalculate game aggregate ratings
        self.internal_repo
            .recalculate_game_ratings(game_id)
            .await?;

        Ok(saved)
    }
}

impl From<InternalRepositoryError> for UseCaseError {
    fn from(e: InternalRepositoryError) -> Self {
        match e {
            InternalRepositoryError::Internal(e) => UseCaseError::Internal(e),
        }
    }
}