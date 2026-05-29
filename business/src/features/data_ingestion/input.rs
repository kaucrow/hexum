use async_trait::async_trait;
use thiserror::Error;

#[async_trait]
pub trait UseCase: Send + Sync + 'static {
    async fn sync_data(&self) -> Result<(), UseCaseError>;
}

#[derive(Error, Debug)]
pub enum UseCaseError {
    /// Unexpected internal error.
    #[error("Data ingestion service: {0}.")]
    Internal(String),
}