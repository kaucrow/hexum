use std::str::FromStr;

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
impl Repository for PostgresAdapter {
    async fn insert_application(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<CriticApplication, RepositoryError> {
        let res: Result<_, LocalError> = async {
            let row = sqlx::query_as::<_, CriticApplicationDbRow>(
                sql(&QUERIES.critic_application.insert),
            )
            .bind(id)
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;

            Ok(CriticApplication::try_from(row)?)
        }
        .await;

        res.map_err(Into::into)
    }

    async fn get_application_by_user_id(
        &self,
        user_id: &Uuid,
    ) -> Result<Option<CriticApplication>, RepositoryError> {
        let res: Result<_, LocalError> = async {
            let app = sqlx::query_as::<_, CriticApplicationDbRow>(
                sql(&QUERIES.critic_application.get_by_user_id),
            )
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?
            .map(|row| CriticApplication::try_from(row))
            .transpose()?;

            Ok(app)
        }
        .await;

        res.map_err(Into::into)
    }

    async fn get_pending_applications(&self) -> Result<Vec<CriticApplication>, RepositoryError> {
        let res: Result<_, LocalError> = async {
            let rows = sqlx::query_as::<_, CriticApplicationWithUserDbRow>(
                sql(&QUERIES.critic_application.get_pending),
            )
            .fetch_all(&self.pool)
            .await?;

            let apps: Result<Vec<_>, _> = rows
                .into_iter()
                .map(|row| CriticApplication::try_from(row))
                .collect();

            apps
        }
        .await;

        res.map_err(Into::into)
    }

    async fn get_application_by_id(
        &self,
        id: &Uuid,
    ) -> Result<Option<CriticApplication>, RepositoryError> {
        let res: Result<_, LocalError> = async {
            let app = sqlx::query_as::<_, CriticApplicationDbRow>(
                sql(&QUERIES.critic_application.get_by_id),
            )
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .map(|row| CriticApplication::try_from(row))
            .transpose()?;

            Ok(app)
        }
        .await;

        res.map_err(Into::into)
    }

    async fn update_application_status(
        &self,
        id: &Uuid,
        status: &ApplicationStatus,
        reviewer_id: &Uuid,
    ) -> Result<(), RepositoryError> {
        let res: Result<_, LocalError> = async {
            sqlx::query(sql(&QUERIES.critic_application.update_status))
                .bind(id)
                .bind(status.to_string())
                .bind(reviewer_id)
                .execute(&self.pool)
                .await?;

            Ok(())
        }
        .await;

        res.map_err(Into::into)
    }

    async fn add_user_role(&self, user_id: &Uuid, role: &str) -> Result<(), RepositoryError> {
        let res: Result<_, LocalError> = async {
            sqlx::query(sql(&QUERIES.user.add_role))
                .bind(user_id)
                .bind(role)
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

impl From<LocalError> for RepositoryError {
    fn from(e: LocalError) -> Self {
        match e {
            LocalError::Sqlx(e) => RepositoryError::Internal(e.to_string()),
            LocalError::Parse(s) => RepositoryError::Internal(s),
        }
    }
}

#[derive(FromRow)]
pub struct CriticApplicationDbRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub status: String,
    pub applied_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub reviewed_by: Option<Uuid>,
}

impl TryFrom<CriticApplicationDbRow> for CriticApplication {
    type Error = LocalError;

    fn try_from(row: CriticApplicationDbRow) -> Result<Self, Self::Error> {
        let status = ApplicationStatus::from_str(&row.status)
            .map_err(|e| LocalError::Parse(format!("Invalid application status '{}': {}", row.status, e)))?;

        Ok(Self {
            id: row.id,
            user_id: row.user_id,
            status,
            applied_at: row.applied_at,
            reviewed_at: row.reviewed_at,
            reviewed_by: row.reviewed_by,
            username: None,
            email: None,
        })
    }
}

#[derive(FromRow)]
pub struct CriticApplicationWithUserDbRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub status: String,
    pub applied_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub reviewed_by: Option<Uuid>,
    pub username: String,
    pub email: String,
}

impl TryFrom<CriticApplicationWithUserDbRow> for CriticApplication {
    type Error = LocalError;

    fn try_from(row: CriticApplicationWithUserDbRow) -> Result<Self, Self::Error> {
        let status = ApplicationStatus::from_str(&row.status)
            .map_err(|e| LocalError::Parse(format!("Invalid application status '{}': {}", row.status, e)))?;

        Ok(Self {
            id: row.id,
            user_id: row.user_id,
            status,
            applied_at: row.applied_at,
            reviewed_at: row.reviewed_at,
            reviewed_by: row.reviewed_by,
            username: Some(row.username),
            email: Some(row.email),
        })
    }
}