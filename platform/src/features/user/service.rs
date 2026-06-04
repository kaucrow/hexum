use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    prelude::*,
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

    async fn generate_verification_code(
        &self,
        payload: &str,
        ttl_seconds: u64,
    ) -> Result<String, UseCaseError> {
        let mut attempts = 0;
        let max_attempts = 5;

        loop {
            attempts += 1;
            let code = self.security.generate_verification_token();

            match self.verification.store_verification_token(payload, &code, ttl_seconds).await {
                Ok(()) => break Ok(code),
                Err(verification::PortError::CodeInUse) if attempts < max_attempts => {
                    // Code is already in use. Retry with a new code
                    continue;
                }
                Err(verification::PortError::CodeInUse) => {
                    break Err(UseCaseError::Internal(
                        "Failed to generate a unique verification code after multiple attempts".into(),
                    ));
                }
                Err(e) => {
                    break Err(UseCaseError::Internal(e.to_string()));
                }
            }
        }
    }
}

#[async_trait]
impl UseCase for Service {
    async fn get_user_by_id(&self, id: &Uuid) -> Result<Option<User>, UseCaseError> {
        let user = self.user_repo.get_user_by_id(id).await?;

        Ok(user)
    }

    async fn change_user_email(&self, new_email: &str) -> Result<(), UseCaseError> {
        let user_new_email = EmailAddress::new(new_email.to_string())?;

        // Check if the new email is already in use by another user
        if self.user_repo.get_user_by_email(&user_new_email).await?.is_some() {
            return Err(UseCaseError::Conflict(ConflictError::EmailInUse));
        }

        let verification_code = self.generate_verification_code(new_email, 1800).await?;

        self.email.send_verification_email(
            &user_new_email,
            &verification_code,
            email::VerificationContext::EmailChange,
        ).await?;

        Ok(())
    }

    async fn verify_user_email_change(&self, user_id: &Uuid, code: &str) -> Result<(), UseCaseError> {
        let new_email_str = self.verification.consume_verification_token(code).await?;
        let new_email = EmailAddress::new(new_email_str)?;

        // Check if the new email is already in use by another user
        if let Some(_) = self.user_repo.get_user_by_email(&new_email).await? {
            return Err(UseCaseError::Conflict(ConflictError::EmailInUse));
        }

        self.user_repo.update_user_email(user_id, &new_email).await?;

        Ok(())
    }

    async fn delete_user(&self, user_id: &Uuid) -> Result<Option<Uuid>, UseCaseError> {
        let deleted_user_id = self.user_repo.delete_user_by_id(user_id).await?;

        Ok(deleted_user_id)
    }

    async fn register_user(&self, user: User, passwd: &str) -> Result<(), UseCaseError> {
        let user_id = user.id.clone();
        let user_email = user.email.clone();

        let passwd = Password::new(passwd.to_string())?;
        let passwd_hash = self.security.hash_password(&passwd)?;
        let auth = UserAuthenticator::new_local(user_id, passwd_hash);

        self.user_repo.add_new_user(user).await?;
        self.user_repo.add_authenticator(auth).await?;

        let verification_code = match self.generate_verification_code(&user_id.to_string(), 1800).await {
            Ok(code) => code,
            Err(e) => {
                // Rollbacks the user creation if the verification code generation fails
                let _ = self.user_repo.delete_user_by_id(&user_id).await;
                return Err(e);
            }
        };

        let email_result = self.email.send_verification_email(
            &user_email,
            &verification_code,
            email::VerificationContext::AccountRegistration,
        ).await;

        if let Err(e) = email_result {
            self.user_repo.delete_user_by_id(&user_id).await?;
            return Err(UseCaseError::Internal(e.to_string()))
        }

        Ok(())
    }

    async fn verify_user_account(&self, code: &str) -> Result<(), UseCaseError> {
        let user_id_str = self.verification.consume_verification_token(code).await?;
        let user_id = Uuid::parse_str(&user_id_str).map_err(|_|
            UseCaseError::Internal("Invalid User ID format from verification token payload.".to_string())
        )?;

        self.user_repo.verify_local_auth_by_user_id(&user_id).await?;

        Ok(())
    }
}

impl From<RepositoryError> for UseCaseError {
    fn from(e: RepositoryError) -> Self {
        match e {
            RepositoryError::Conflict(c) => UseCaseError::Conflict(c),
            RepositoryError::Internal(s) => UseCaseError::Internal(s),
        }
    }
}

impl From<security::PortError> for UseCaseError {
    fn from(e: security::PortError) -> Self {
        match e {
            security::PortError::TokenVerificationFailed => {
                UseCaseError::Internal(e.to_string())
            }
            security::PortError::Internal(s) => UseCaseError::Internal(s),
        }
    }
}

impl From<email::PortError> for UseCaseError {
    fn from(e: email::PortError) -> Self {
        UseCaseError::Internal(e.to_string())
    }
}

impl From<verification::PortError> for UseCaseError {
    fn from(e: verification::PortError) -> Self {
        match e {
            verification::PortError::VerificationTokenInvalid => UseCaseError::VerificationTokenInvalid,
            verification::PortError::CodeInUse => UseCaseError::Internal(e.to_string()),
            verification::PortError::Internal(s) => UseCaseError::Internal(s),
        }
    }
}