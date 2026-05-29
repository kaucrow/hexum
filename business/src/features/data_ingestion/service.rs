use std::sync::Arc;

use async_trait::async_trait;

use super::*;

#[derive(Clone)]
pub struct Service {
    local_repo: Arc<dyn LocalRepository>,
    external_repo: Arc<dyn ExternalRepository>,
}

impl Service {
    pub fn new(
        local_repo: Arc<dyn LocalRepository>,
        external_repo: Arc<dyn ExternalRepository>,
    ) -> Self {
        Self { local_repo, external_repo }
    }
}

#[async_trait]
impl UseCase for Service {
    async fn sync_data(&self) -> Result<(), UseCaseError> {
        for c in 'a'..='z' {
            let recipes = self.external_repo.get_recipes_by_first_letter(c).await?;

            self.local_repo.sync_recipes(recipes).await?;
        }

        Ok(())
    }
}

impl From<LocalRepositoryError> for UseCaseError {
    fn from(e: LocalRepositoryError) -> Self {
        Self::Internal(e.to_string())
    }
}

impl From<ExternalRepositoryError> for UseCaseError {
    fn from(e: ExternalRepositoryError) -> Self {
        Self::Internal(e.to_string())
    }
}