use async_trait::async_trait;
use thiserror::Error;

#[async_trait]
pub trait UseCase: Send + Sync + 'static {
    async fn check_db_health(&self) -> Result<(), UseCaseError>;
}

#[derive(Error, Debug)]
pub enum UseCaseError {
    #[error("Base service: {0}")]
    Internal(String),
}