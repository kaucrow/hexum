use async_trait::async_trait;
use thiserror::Error;

use crate::prelude::*;
use super::{CriticApplication, ApplicationStatus};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Repository: Send + Sync + 'static {
    async fn insert_application(&self, id: Uuid, user_id: Uuid)
        -> Result<CriticApplication, RepositoryError>;

    async fn get_application_by_user_id(
        &self,
        user_id: &Uuid,
    ) -> Result<Option<CriticApplication>, RepositoryError>;

    async fn get_pending_applications(&self) -> Result<Vec<CriticApplication>, RepositoryError>;

    async fn get_application_by_id(
        &self,
        id: &Uuid,
    ) -> Result<Option<CriticApplication>, RepositoryError>;

    async fn update_application_status(
        &self,
        id: &Uuid,
        status: &ApplicationStatus,
        reviewer_id: &Uuid,
    ) -> Result<(), RepositoryError>;

    /// Adds a role to a user (e.g., 'Critic').
    async fn add_user_role(&self, user_id: &Uuid, role: &str) -> Result<(), RepositoryError>;
}

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Critic repository: {0}")]
    Internal(String),
}