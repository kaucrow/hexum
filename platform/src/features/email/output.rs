use async_trait::async_trait;
use thiserror::Error;

use crate::features::user;

#[async_trait]
pub trait Port: Send + Sync + 'static {
    async fn send_verification_email(&self, to: &user::EmailAddress, token: &str) -> Result<(), PortError>;
}

#[derive(Error, Debug)]
pub enum PortError {
    #[error("Email: {0}")]
    Internal(String)
}