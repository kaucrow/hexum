use crate::prelude::*;
use super::CriticApplication;

#[async_trait]
pub trait UseCase: Send + Sync + 'static {
    /// A BasicUser applies to become a Critic.
    async fn apply_for_critic(&self, user_id: &Uuid) -> Result<CriticApplication, UseCaseError>;

    /// Admin lists all pending applications.
    async fn list_pending_applications(&self) -> Result<Vec<CriticApplication>, UseCaseError>;

    /// Admin approves an application, granting the Critic role.
    async fn approve_application(
        &self,
        application_id: &Uuid,
        reviewer_id: &Uuid,
    ) -> Result<(), UseCaseError>;

    /// Admin rejects an application.
    async fn reject_application(
        &self,
        application_id: &Uuid,
        reviewer_id: &Uuid,
    ) -> Result<(), UseCaseError>;
}

#[derive(Error, Debug)]
pub enum UseCaseError {
    #[error("You have already applied for the Critic role.")]
    AlreadyApplied,

    #[error("Application not found.")]
    ApplicationNotFound,

    #[error("Application has already been reviewed.")]
    AlreadyReviewed,

    #[error("Critic service: {0}")]
    Internal(String),
}