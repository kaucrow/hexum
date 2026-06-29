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
    async fn apply_for_critic(&self, user_id: &Uuid) -> Result<CriticApplication, UseCaseError> {
        // Check if user already has a pending or approved application
        let existing = self.repo.get_application_by_user_id(user_id).await
            .map_err(|e| UseCaseError::Internal(e.to_string()))?;

        if let Some(app) = existing {
            match app.status {
                ApplicationStatus::Pending => return Err(UseCaseError::AlreadyApplied),
                ApplicationStatus::Approved => return Err(UseCaseError::AlreadyApplied),
                // If previously rejected, allow re-application
                ApplicationStatus::Rejected => {}
            }
        }

        let application_id = Uuid::new_v4();
        let application = self.repo
            .insert_application(application_id, *user_id)
            .await
            .map_err(|e| UseCaseError::Internal(e.to_string()))?;

        Ok(application)
    }

    async fn list_pending_applications(&self) -> Result<Vec<CriticApplication>, UseCaseError> {
        let applications = self.repo
            .get_pending_applications()
            .await
            .map_err(|e| UseCaseError::Internal(e.to_string()))?;

        Ok(applications)
    }

    async fn approve_application(
        &self,
        application_id: &Uuid,
        reviewer_id: &Uuid,
    ) -> Result<(), UseCaseError> {
        let application = self.repo
            .get_application_by_id(application_id)
            .await
            .map_err(|e| UseCaseError::Internal(e.to_string()))?
            .ok_or(UseCaseError::ApplicationNotFound)?;

        if application.status != ApplicationStatus::Pending {
            return Err(UseCaseError::AlreadyReviewed);
        }

        // Update application status to approved
        self.repo
            .update_application_status(application_id, &ApplicationStatus::Approved, reviewer_id)
            .await
            .map_err(|e| UseCaseError::Internal(e.to_string()))?;

        // Grant the Critic role to the user
        self.repo
            .add_user_role(&application.user_id, "Critic")
            .await
            .map_err(|e| UseCaseError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn reject_application(
        &self,
        application_id: &Uuid,
        reviewer_id: &Uuid,
    ) -> Result<(), UseCaseError> {
        let application = self.repo
            .get_application_by_id(application_id)
            .await
            .map_err(|e| UseCaseError::Internal(e.to_string()))?
            .ok_or(UseCaseError::ApplicationNotFound)?;

        if application.status != ApplicationStatus::Pending {
            return Err(UseCaseError::AlreadyReviewed);
        }

        self.repo
            .update_application_status(application_id, &ApplicationStatus::Rejected, reviewer_id)
            .await
            .map_err(|e| UseCaseError::Internal(e.to_string()))?;

        Ok(())
    }
}