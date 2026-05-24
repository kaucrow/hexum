use async_trait::async_trait;
use thiserror::Error;

#[async_trait]
pub trait Port: Send + Sync + 'static {
    async fn ping_db(&self) -> Result<(), PortError>;
}

#[derive(Error, Debug)]
pub enum PortError {
    #[error("Base: {0}")]
    Internal(String),
}