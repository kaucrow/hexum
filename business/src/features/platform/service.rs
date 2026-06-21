use crate::prelude::*;
use super::*;

#[derive(Clone)]
pub struct Service {
    internal_repo: Arc<dyn InternalRepository>,
    external_repo: Arc<dyn ExternalRepository>,
}

impl Service {
    pub fn new(
        internal_repo: Arc<dyn InternalRepository>,
        external_repo: Arc<dyn ExternalRepository>,
    ) -> Self {
        Self { internal_repo, external_repo }
    }
}

#[async_trait]
impl UseCase for Service {
    async fn get_platforms(&self) -> Result<Vec<Platform>, UseCaseError> {
        self.internal_repo
            .get_platforms()
            .await
            .map_err(UseCaseError::from)
    }

    async fn sync_db_platforms(&self) -> Result<(), UseCaseError> {
        let platforms = self.external_repo
            .get_platforms()
            .await
            .map_err(UseCaseError::from)?;

        self.internal_repo
            .upsert_platforms(&platforms)
            .await
            .map_err(UseCaseError::from)?;

        info!("Synced {} platforms from external repository.", platforms.len());

        Ok(())
    }
}