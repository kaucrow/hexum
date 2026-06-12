use async_trait::async_trait;
use rand::distr::{Alphanumeric, SampleString};

use crate::{
    prelude::*,
    features::{
        user::{self, User, AuthProvider},
        session::{self, SessionPayload},
        security, oauth
    },
};
use super::*;

/// A well-formed but never-matching argon2id hash used when the user
/// does not exist, to ensure constant-time password verification and
/// prevent time-based attacks that could reveal valid usernames.
const DUMMY_ARGON2_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$AAAAAAAAAAAAAAAAAAAAAA$AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

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

    /// Resolves a user by identity (username or email) and returns the
    /// user along with their local authenticator (if any) and the password
    /// hash to verify against.
    ///
    /// When the user is not found, returns `None` for both user and
    /// authenticator. The caller must then verify against [`DUMMY_ARGON2_HASH`]
    /// to maintain constant-time behaviour.
    async fn resolve_local_user(
        &self,
        identity: &str,
    ) -> Result<(Option<User>, Option<user::UserAuthenticator>), UseCaseError> {
        // Try username first
        let user = if let Some(u) = self.user_repo.get_user_by_username(identity).await? {
            Some(u)
        } else {
            // Try email
            if let Ok(email) = user::EmailAddress::new(identity.to_string()) {
                self.user_repo.get_user_by_email(&email).await?
            } else {
                None
            }
        };

        let authenticator = match &user {
            Some(u) => self.user_repo
                .get_authenticator(&u.id, AuthProvider::Local)
                .await?,
            None => None,
        };

        Ok((user, authenticator))
    }

    async fn resolve_and_login(
        &self,
        email_str: String,
        external_id: String,
        provider: AuthProvider,
    ) -> Result<AuthTokens, UseCaseError> {
        let email = user::EmailAddress::new(email_str)
            .map_err(|e| UseCaseError::Internal(e.to_string()))?;

        let user = match self.user_repo.get_user_by_email(&email).await? {
            Some(existing_user) => {
                if !existing_user.is_active {
                    return Err(UseCaseError::UserInactive);
                }
                // Link the provider if this is the user's first time using this OAuth provider for login
                self.ensure_provider_linked(&existing_user.id, provider.clone(), external_id).await?;
                existing_user
            }
            None => {
                // Completely new user
                self.register_oauth_user(email, provider.clone(), external_id).await?
            }
        };

        self.issue_session(
            SessionPayload {
                user_id: user.id,
                roles: user.roles,
                provider,
            }
        ).await
    }

    async fn ensure_provider_linked(
        &self,
        user_id: &Uuid,
        provider: AuthProvider,
        provider_id: String,
    ) -> Result<(), UseCaseError> {
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
        provider: AuthProvider,
        provider_id: String,
    ) -> Result<User, UseCaseError> {
        let suffix = Alphanumeric.sample_string(&mut rand::rng(), 6);
        let temp_username = format!("user{}", suffix);

        let user = User::new(&temp_username, &email.as_str()).map_err(|e| UseCaseError::Internal(e.to_string()))?;
        let auth = user::UserAuthenticator::new_oauth(user.id, provider, provider_id);

        self.user_repo.add_new_user(user.clone()).await?;
        self.user_repo.add_authenticator(auth).await?;

        Ok(user)
    }

    async fn issue_session(&self, payload: SessionPayload) -> Result<AuthTokens, UseCaseError> {
        let access_token = self.security.generate_access_token(payload.clone())?;
        let refresh_token = self.security.generate_refresh_token();

        self.session
            .store_session(&refresh_token, payload, 7)
            .await?;

        Ok(AuthTokens { access_token, refresh_token })
    }
}

#[async_trait]
impl UseCase for Service {
    async fn login_user(&self, identity: &str, passwd: &str) -> Result<AuthTokens, UseCaseError> {
        // Resolve user + local authenticator (or None for both)
        let (user_opt, auth_opt) = self.resolve_local_user(identity).await?;

        // Determine the hash to verify, real or dummy.
        // Always run an argon2 verification so an attacker cannot
        // distinguish "user not found" from "wrong password" via response time.
        let hash_to_verify = auth_opt
            .as_ref()
            .and_then(|a| a.hashed_passwd.as_deref())
            .unwrap_or(DUMMY_ARGON2_HASH);

        let password_ok = self.security.verify_password(passwd, hash_to_verify);

        // Combine all checks into a single decision
        match (user_opt, auth_opt, password_ok) {
            (Some(user), Some(auth), true) => {
                if !user.is_active {
                    return Err(UseCaseError::InvalidCredentials);
                }
                if let Some(false) = auth.is_verified {
                    return Err(UseCaseError::InvalidCredentials);
                }

                self.issue_session(
                    SessionPayload {
                        user_id: user.id,
                        roles: user.roles,
                        provider: AuthProvider::Local,
                    }
                ).await
            }
            _ => Err(UseCaseError::InvalidCredentials),
        }
    }

    async fn login_user_via_google(&self, code: &str) -> Result<AuthTokens, UseCaseError> {
        let google_user = self.oauth
            .get_google_user_info_by_code(code)
            .await
            .map_err(|e| UseCaseError::Internal(format!("Google Auth failed: {:?}", e)))?;

        self.resolve_and_login(
            google_user.email,
            google_user.external_id,
            AuthProvider::Google,
        )
        .await
    }

    async fn login_user_via_github(&self, code: &str) -> Result<AuthTokens, UseCaseError> {
        let github_user = self.oauth
            .get_github_user_info_by_code(code)
            .await
            .map_err(|e| UseCaseError::Internal(format!("GitHub Auth failed: {:?}", e)))?;

        self.resolve_and_login(
            github_user.email,
            github_user.external_id.to_string(),
            AuthProvider::GitHub,
        )
        .await
    }

    async fn verify_session(&self, access_token: &str) -> Result<SessionPayload, UseCaseError> {
        let payload = self.security.verify_access_token(access_token)?;
        Ok(payload)
    }

    async fn refresh_session(&self, refresh_token: &str) -> Result<AuthTokens, UseCaseError> {
        // Consume the session
        let session = self.session
            .consume_session(refresh_token)
            .await?
            .ok_or(UseCaseError::InvalidRefreshToken)?;

        // Fetch user
        let user = self.user_repo.get_user_by_id(&session.user_id)
            .await?
            .ok_or(UseCaseError::UserNotFound)?;

        if !user.is_active {
            return Err(UseCaseError::UserInactive)
        }

        let auth_tokens = self.issue_session(session).await?;

        Ok(auth_tokens)
    }

    async fn logout_user(&self, refresh_token: &str) -> Result<(), UseCaseError> {
       self.session
            .consume_session(refresh_token)
            .await?
            .ok_or(UseCaseError::InvalidRefreshToken)?;

        Ok(())
    }
}

impl From<user::RepositoryError> for UseCaseError {
    fn from(e: user::RepositoryError) -> Self {
        UseCaseError::Internal(e.to_string())
    }
}

impl From<session::PortError> for UseCaseError {
    fn from(e: session::PortError) -> Self {
        UseCaseError::Internal(e.to_string())
    }
}

impl From<security::PortError> for UseCaseError {
    fn from(e: security::PortError) -> Self {
        match e {
            security::PortError::TokenVerificationFailed => {
                UseCaseError::InvalidAccessToken(e.to_string())
            }
            security::PortError::Internal(s) => UseCaseError::Internal(s),
        }
    }
}

impl From<oauth::PortError> for UseCaseError {
    fn from(e: oauth::PortError) -> Self {
        match e {
            oauth::PortError::InvalidCode => UseCaseError::InvalidOAuthCode(e.to_string()),
            oauth::PortError::NetworkError(s) => UseCaseError::Internal(s),
            oauth::PortError::ParseError => UseCaseError::Internal(e.to_string()),
            oauth::PortError::Internal(s) => UseCaseError::Internal(s),
        }
    }
}