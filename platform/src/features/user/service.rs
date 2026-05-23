use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::{security, email, verification},
};
use super::*;

#[derive(Clone)]
pub struct Service {
    user_repo: Arc<dyn Repository>,
    verification: Arc<dyn verification::Port>,
    security: Arc<dyn security::Port>,
    email: Arc<dyn email::Port>,
}

impl Service {
    pub fn new(
        user_repo: Arc<dyn Repository>,
        verification: Arc<dyn verification::Port>,
        security: Arc<dyn security::Port>,
        email: Arc<dyn email::Port>,
    ) -> Self {
        Self { user_repo, verification, security, email }
    }
}

#[async_trait]
impl UseCase for Service {
    async fn register_user(&self, user: User, passwd: &str) -> Result<(), UseCaseError> {
        let user_id = user.id.clone();
        let user_email = user.email.clone();

        let passwd_hash = self.security.hash(passwd)?;
        let auth = UserAuthenticator::new_local(user_id, passwd_hash);

        self.user_repo.add_new_user(user).await?;
        self.user_repo.add_authenticator(auth).await?;

        let verification_token = self.security.generate_verification_token();

        self.verification.store_verification_token(&user_id, &verification_token, 1800).await?;

        let email_result = self.email.send_verification_email(&user_email, &verification_token).await;

        if let Err(e) = email_result {
            self.user_repo.delete_user_by_id(&user_id).await?;
            return Err(UseCaseError::Internal(e.to_string()))
        }

        Ok(())
    }

    async fn verify_user_account(&self, token: &str) -> Result<(), UseCaseError> {
        let user_id = self.verification.consume_verification_token(token).await?;
        self.user_repo.verify_local_auth_by_user_id(&user_id).await?;

        Ok(())
    }
}

impl From<RepositoryError> for UseCaseError {
    fn from(e: RepositoryError) -> Self {
        match e {
            RepositoryError::UsernameInUse => UseCaseError::UsernameInUse,
            RepositoryError::EmailInUse => UseCaseError::EmailInUse,
            _ => UseCaseError::Internal(e.to_string()),
        }
    }
}

impl From<security::PortError> for UseCaseError {
    fn from(e: security::PortError) -> Self {
        match e {
            _ => UseCaseError::Internal(e.to_string()),
        }
    }
}

impl From<email::PortError> for UseCaseError {
    fn from(e: email::PortError) -> Self {
        match e {
            _ => UseCaseError::Internal(e.to_string()),
        }
    }
}

impl From<verification::PortError> for UseCaseError {
    fn from(e: verification::PortError) -> Self {
        match e {
            _ => UseCaseError::Internal(e.to_string()),
        }
    }
}