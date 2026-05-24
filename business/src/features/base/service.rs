use std::sync::Arc;

use async_trait::async_trait;

use crate::features::base;
use super::*;

#[derive(Clone)]
pub struct Service {
    base: Arc<dyn base::Port>,
}

impl Service {
    pub fn new(
        base: Arc<dyn base::Port>,
    ) -> Self {
        Self { base }
    }
}

#[async_trait]
impl UseCase for Service {
    async fn check_db_health(&self) -> Result<(), UseCaseError> {
        Ok(self.base.ping_db().await?)
    }
}

impl From<base::PortError> for UseCaseError {
    fn from(e: base::PortError) -> Self {
        match e {
            _ => UseCaseError::Internal(e.to_string()),
        }
    }
}