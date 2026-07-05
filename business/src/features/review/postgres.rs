use std::str::FromStr;

use async_trait::async_trait;
use thiserror::Error;

use crate::{
    prelude::*,
    postgres::*,
};
use super::*;

#[derive(Clone)]
pub struct PostgresAdapter {
    pub pool: PgPool,
}

impl PostgresAdapter {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl InternalRepository for PostgresAdapter {
    async fn get_game_reviews(&self, game_id: &Uuid) -> Result<Vec<Review>, InternalRepositoryError> {
        let res: Result<_, LocalError> = async {
            let reviews: Result<Vec<Review>, _> = sqlx::query_as::<_, ReviewDbRow>(
                sql(&QUERIES.review.get_all_by_game),
            )
            .bind(game_id)
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|row| Review::try_from(row))
            .collect();

            Ok(reviews?)
        }
        .await;

        res.map_err(Into::into)
    }

    async fn get_review_by_user_and_game(
        &self,
        user_id: &Uuid,
        game_id: &Uuid,
    ) -> Result<Option<Review>, InternalRepositoryError> {
        let res: Result<_, LocalError> = async {
            let review = sqlx::query_as::<_, ReviewDbRow>(
                sql(&QUERIES.review.get_by_user_and_game),
            )
            .bind(user_id)
            .bind(game_id)
            .fetch_optional(&self.pool)
            .await?
            .map(|row| Review::try_from(row))
            .transpose()?;

            Ok(review)
        }
        .await;

        res.map_err(Into::into)
    }

    async fn upsert_review(&self, review: &Review) -> Result<Review, InternalRepositoryError> {
        let res: Result<_, LocalError> = async {
            let row = sqlx::query_as::<_, ReviewDbRow>(
                sql(&QUERIES.review.upsert),
            )
            .bind(review.id)
            .bind(review.user_id)
            .bind(review.game_id)
            .bind(review.review_type.to_string())
            .bind(review.rating)
            .bind(&review.title)
            .bind(&review.content)
            .fetch_one(&self.pool)
            .await?;

            Ok(Review::try_from(row)?)
        }
        .await;

        res.map_err(Into::into)
    }

    async fn recalculate_game_ratings(&self, game_id: &Uuid) -> Result<(), InternalRepositoryError> {
        let res: Result<_, LocalError> = async {
            sqlx::query(sql(&QUERIES.review.recalculate_game_ratings))
                .bind(game_id)
                .execute(&self.pool)
                .await?;

            Ok(())
        }
        .await;

        res.map_err(Into::into)
    }
}

#[derive(Error, Debug)]
pub enum LocalError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("{0}")]
    Parse(String),
}

impl From<LocalError> for InternalRepositoryError {
    fn from(e: LocalError) -> Self {
        match e {
            LocalError::Sqlx(e) => InternalRepositoryError::Internal(e.to_string()),
            LocalError::Parse(s) => InternalRepositoryError::Internal(s),
        }
    }
}

#[derive(FromRow)]
pub struct ReviewDbRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub game_id: Uuid,
    pub review_type: String,
    pub rating: i32,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<ReviewDbRow> for Review {
    type Error = LocalError;

    fn try_from(row: ReviewDbRow) -> Result<Self, Self::Error> {
        let review_type = ReviewType::from_str(&row.review_type)
            .map_err(|e| LocalError::Parse(format!("Invalid review type '{}': {}", row.review_type, e)))?;

        Ok(Self {
            id: row.id,
            user_id: row.user_id,
            game_id: row.game_id,
            review_type,
            rating: row.rating,
            title: row.title,
            content: row.content,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}