use std::sync::Arc;

use async_trait::async_trait;
use platform::features::user;

use crate::prelude::*;
use super::*;

#[derive(Clone)]
pub struct Service {
    repo: Arc<dyn Repository>,
}

impl Service {
    pub fn new(repo: Arc<dyn Repository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl UseCase for Service {
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
        let existing = self.repo
            .get_review_by_user_and_game(user_id, game_id)
            .await
            .map_err(|e| UseCaseError::Internal(e.to_string()))?;

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

        let saved = self.repo
            .upsert_review(&review)
            .await
            .map_err(|e| UseCaseError::Internal(e.to_string()))?;

        // Recalculate game aggregate ratings
        self.repo
            .recalculate_game_ratings(game_id)
            .await
            .map_err(|e| UseCaseError::Internal(e.to_string()))?;

        Ok(saved)
    }
}