use std::sync::Arc;

use async_trait::async_trait;
use rand::distr::{Alphanumeric, SampleString};
use uuid::Uuid;

use crate::{
    features::{user, auth, session, security, oauth},
};
use super::*;

#[derive(Clone)]
pub struct Service {
    user_repo: Arc<dyn user::Repository>,
    session: Arc<dyn session::Port>,
    security: Arc<dyn security::Port>,
    oauth: Arc<dyn oauth::Port>,
}

impl Service {
    pub fn new(
        user_repo: Arc<dyn user::Repository>,
        session: Arc<dyn session::Port>,
        security: Arc<dyn security::Port>,
        oauth: Arc<dyn oauth::Port>,
    ) -> Self {
        Self { user_repo, session, security, oauth }
    }

    async fn resolve_and_login(
        &self,
        email_str: String,
        external_id: String,
        provider: user::AuthProvider,
    ) -> Result<auth::AuthTokens, auth::UseCaseError> {
        let email = user::EmailAddress::new(email_str)
            .map_err(|e| auth::UseCaseError::Internal(e.to_string()))?;

        let user = match self.user_repo.get_user_by_email(&email).await {
            Some(existing_user) => {
                if !existing_user.is_active {
                    return Err(auth::UseCaseError::UserInactive);
                }
                // Link the provider if this is the user's first time using this OAuth provider for login
                self.ensure_provider_linked(&existing_user.id, provider, external_id).await?;
                existing_user
            }
            None => {
                // Completely new user
                self.register_oauth_user(email, provider, external_id).await?
            }
        };

        self.issue_session(&user.id).await
    }

    async fn ensure_provider_linked(
        &self,
        user_id: &Uuid,
        provider: user::AuthProvider,
        provider_id: String,
    ) -> Result<(), auth::UseCaseError> {
        let existing_auth = self.user_repo.get_authenticator(user_id, provider.clone()).await?;

        if existing_auth.is_none() {
            let new_auth = user::UserAuthenticator::new_oauth(*user_id, provider, provider_id);
            self.user_repo.add_authenticator(new_auth).await?;
        }

        Ok(())
    }

    async fn register_oauth_user(
        &self,
        email: user::EmailAddress,
        provider: user::AuthProvider,
        provider_id: String,
    ) -> Result<user::User, auth::UseCaseError> {
        let suffix = Alphanumeric.sample_string(&mut rand::rng(), 6);
        let temp_username = format!("user{}", suffix);

        let user = user::User::new(&temp_username, &email.as_str()).map_err(|e| auth::UseCaseError::Internal(e.to_string()))?;
        let auth = user::UserAuthenticator::new_oauth(user.id, provider, provider_id);

        self.user_repo.add_new_user(user.clone()).await?;
        self.user_repo.add_authenticator(auth).await?;

        Ok(user)
    }

    async fn issue_session(&self, user_id: &Uuid) -> Result<auth::AuthTokens, auth::UseCaseError> {
        let access_token = self.security.generate_access_token(user_id)?;
        let refresh_token = self.security.generate_refresh_token();

        self.session
            .store_session(&refresh_token, user_id, 7)
            .await?;

        Ok(auth::AuthTokens { access_token, refresh_token })
    }
}

#[async_trait]
impl auth::UseCase for Service {
    async fn login_user(&self, identity: &str, passwd: &str) -> Result<auth::AuthTokens, auth::UseCaseError> {
        let user = if let Some(u) = self.user_repo.get_user_by_username(identity).await {
            u
        } else {
            // If the identity is not a username, try parsing is as email.
            // If it's not a valid email format, we stop here.
            let email = user::EmailAddress::new(identity.to_string())
                .or(Err(auth::UseCaseError::UserNotFound))?;

            self.user_repo.get_user_by_email(&email).await
                .ok_or(auth::UseCaseError::UserNotFound)?
        };

        if !user.is_active {
            return Err(auth::UseCaseError::UserInactive);
        }

        let local_authenticator = self.user_repo
            .get_authenticator(&user.id, user::AuthProvider::Local)
            .await?
            .ok_or(auth::UseCaseError::UserNotFound)?;

        if let Some(is_verified) = local_authenticator.is_verified {
            if !is_verified {
                return Err(auth::UseCaseError::UserNotVerified);
            }
        }

        let passwd_hash = local_authenticator.passwd
            .ok_or(auth::UseCaseError::Internal("User with local auth has no password set.".to_string()))?;

        if !self.security.verify_password(&passwd, &passwd_hash) {
            return Err(auth::UseCaseError::InvalidPassword);
        }

        let auth_tokens = self.issue_session(&user.id).await?;

        Ok(auth_tokens)
    }

    async fn login_user_via_google(&self, code: &str) -> Result<AuthTokens, auth::UseCaseError> {
        let google_user = self.oauth
            .get_google_user_info_by_code(code)
            .await
            .map_err(|e| auth::UseCaseError::Internal(format!("Google Auth failed: {:?}", e)))?;

        self.resolve_and_login(
            google_user.email,
            google_user.external_id,
            user::AuthProvider::Google,
        )
        .await
    }

    async fn login_user_via_github(&self, code: &str) -> Result<AuthTokens, auth::UseCaseError> {
        let github_user = self.oauth
            .get_github_user_info_by_code(code)
            .await
            .map_err(|e| auth::UseCaseError::Internal(format!("GitHub Auth failed: {:?}", e)))?;

        self.resolve_and_login(
            github_user.email,
            github_user.external_id.to_string(),
            user::AuthProvider::GitHub,
        )
        .await
    }

    async fn verify_user(&self, access_token: &str) -> Result<user::User, auth::UseCaseError> {
        let user_id = self.security.verify_access_token(access_token)?;

        match self.user_repo.get_user_by_id(&user_id).await {
            Some(user) => {
                if !user.is_active {
                    return Err(auth::UseCaseError::UserInactive)
                }

                Ok(user)
            },
            None => Err(auth::UseCaseError::UserNotFound)
        }
    }

    async fn refresh_session(&self, refresh_token: &str) -> Result<AuthTokens, auth::UseCaseError> {
        // Consume the session
        let user_id = self.session
            .consume_session(refresh_token)
            .await?
            .ok_or(auth::UseCaseError::InvalidRefreshToken)?;

        // Fetch user
        let user = self.user_repo.get_user_by_id(&user_id)
            .await
            .ok_or(auth::UseCaseError::UserNotFound)?;

        if !user.is_active {
            return Err(auth::UseCaseError::UserInactive)
        }

        let auth_tokens = self.issue_session(&user_id).await?;

        Ok(auth_tokens)
    }

    async fn logout_user(&self, refresh_token: &str) -> Result<(), auth::UseCaseError> {
       self.session
            .consume_session(refresh_token)
            .await?
            .ok_or(auth::UseCaseError::InvalidRefreshToken)?;

        Ok(())
    }
}

impl From<user::RepositoryError> for auth::UseCaseError {
    fn from(e: user::RepositoryError) -> Self {
        match e {
            _ => auth::UseCaseError::Internal(e.to_string()),
        }
    }
}

impl From<session::PortError> for auth::UseCaseError {
    fn from(e: session::PortError) -> Self {
        match e {
            _ => auth::UseCaseError::Internal(e.to_string()),
        }
    }
}

impl From<security::PortError> for auth::UseCaseError {
    fn from(e: security::PortError) -> Self {
        match e {
            security::PortError::TokenVerificationFailed => auth::UseCaseError::InvalidAccessToken(e.to_string()),
            _ => auth::UseCaseError::Internal(e.to_string()),
        }
    }
}

impl From<oauth::PortError> for auth::UseCaseError {
    fn from(e: oauth::PortError) -> Self {
        match e {
            oauth::PortError::InvalidCode => auth::UseCaseError::InvalidOAuthCode(e.to_string()),
            _ => auth::UseCaseError::Internal(e.to_string()),
        }
    }
}