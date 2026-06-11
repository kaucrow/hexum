use async_trait::async_trait;
use argon2::{
    Argon2,
    PasswordHash,
    PasswordVerifier,
    PasswordHasher,
    password_hash::{SaltString, rand_core::OsRng},
};
use pasetors::{
    Local,
    claims::{Claims, ClaimsValidationRules},
    version4::V4,
    keys::{SymmetricKey, Generate},
    token::UntrustedToken,
    local,
};
use chrono::{Utc, Duration};
use rand::{
    RngExt,
    distr::{Alphanumeric, SampleString},
};
use thiserror::Error;
use uuid::Uuid;
use anyhow::Result;

use crate::{
    prelude::*,
    features::{
        session::SessionPayload,
        user::{self, Role, AuthProvider},
    },
};
use super::*;

#[derive(Clone)]
pub struct PasetoAdapter {
    pub sk: SymmetricKey<V4>,
}

impl PasetoAdapter {
    pub fn new() -> Result<Self> {
        let sk = SymmetricKey::<V4>::generate()?;

        Ok(Self { sk })
    }
}

#[async_trait]
impl Port for PasetoAdapter {
    // Verify Argon2 hash
    fn verify_password(&self, password: &str, hash: &str) -> bool {
        let parsed_hash = match PasswordHash::new(hash) {
            Ok(hash) => hash,
            Err(_) => return false,
        };

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
    }

    // Hash a string with Argon2
    fn hash_password(&self, password: &user::Password) -> Result<String, PortError> {
        let res: Result<_, LocalError> = {
            let salt = SaltString::generate(&mut OsRng);
            let argon2 = Argon2::default();
            let hash = argon2
                .hash_password(password.as_str().as_bytes(), &salt)
                .map_err(LocalError::Argon2)?
                .to_string();

            Ok(hash)
        };

        res.map_err(Into::into)
    }

    // Verify a PASETO v4 access token & return the session payload
    fn verify_access_token(&self, token: &str) -> Result<SessionPayload, PortError> {
        let res: Result<_, LocalError> = {
            let validation_rules = ClaimsValidationRules::new();
            let untrusted_token = UntrustedToken::<Local, V4>::try_from(token)
                .map_err(|_| LocalError::TokenVerificationFailed)?;
            let trusted_token = local::decrypt(&self.sk, &untrusted_token, &validation_rules, None, None)
                .map_err(|_| LocalError::TokenVerificationFailed)?;

            let claims = trusted_token.payload_claims();

            // Extract and parse the user ID
            let user_id_str = claims
            .and_then(|c| c.get_claim("user_id"))
            .and_then(|json_value| json_value.as_str())
            .ok_or(LocalError::TokenVerificationFailed)?;

            let user_id = Uuid::try_parse(user_id_str)
                .map_err(|_| LocalError::TokenVerificationFailed)?;

            // Extract and parse the roles
            let roles_str = claims
                .and_then(|c| c.get_claim("roles"))
                .and_then(|json_value| json_value.as_str())
                .ok_or(LocalError::TokenVerificationFailed)?;

            let roles: Vec<Role> = serde_json::from_str(roles_str)
                .map_err(|_| LocalError::TokenVerificationFailed)?;

            // Extract and parse the auth provider
            let provider_str = claims
                .and_then(|c| c.get_claim("provider"))
                .and_then(|json_value| json_value.as_str())
                .ok_or(LocalError::TokenVerificationFailed)?;

            let provider = AuthProvider::from_str(provider_str)
                .map_err(|_| LocalError::TokenVerificationFailed)?;

            Ok(SessionPayload {
                user_id,
                roles,
                provider,
            })
        };

        res.map_err(Into::into)
    }

    // Generate a PASETO v4 access token
    fn generate_access_token(&self, payload: SessionPayload) -> Result<String, PortError> {
        let res: Result<_, LocalError> = {
            let mut claims =
                Claims::new().map_err(|e| LocalError::Paseto(e))?;

            claims
                .add_additional("user_id", payload.user_id.to_string())
                .map_err(|e| LocalError::Paseto(e))?;

            let roles_json = serde_json::to_string(&payload.roles)
                .map_err(|e| LocalError::Serialization(e))?;
            claims
                .add_additional("roles", roles_json)
                .map_err(|e| LocalError::Paseto(e))?;

            claims
                .add_additional("provider", payload.provider.to_string())
                .map_err(|e| LocalError::Paseto(e))?;

            // Expiration will be 24 hours from current time
            let expiration = Utc::now()
                .checked_add_signed(Duration::hours(24))
                .expect("Timestamp should not overflow");

            // Format the expiration to RFC3339 and set it
            claims
                .expiration(&expiration.to_rfc3339())
                .map_err(|e| LocalError::Paseto(e))?;

            // Encrypt the claims
            let token =
                local::encrypt(&self.sk, &claims, None, None)
                    .map_err(|e| LocalError::Paseto(e))?;

            Ok(token)
        };

        res.map_err(Into::into)
    }

    // Generate a 64-characters long refresh token
    fn generate_refresh_token(&self) -> String {
        Alphanumeric.sample_string(&mut rand::rng(), 64)
    }

    // Generate a 6-digit verification code
    fn generate_verification_token(&self) -> String {
        let code: u32 = rand::rng().random_range(0..1_000_000);
        format!("{code:06}")
    }
}

#[derive(Error, Debug)]
pub enum LocalError {
    #[error("")]
    TokenVerificationFailed,
    #[error(transparent)]
    Paseto(#[from] pasetors::errors::Error),
    #[error(transparent)]
    Uuid(#[from] uuid::Error),
    #[error(transparent)]
    Serialization(#[from] serde_json::Error),
    #[error(transparent)]
    Argon2(#[from] argon2::password_hash::Error),
}

impl From<LocalError> for PortError {
    fn from(e: LocalError) -> Self {
        match e {
            LocalError::TokenVerificationFailed => PortError::TokenVerificationFailed,
            LocalError::Paseto(e) => PortError::Internal(e.to_string()),
            LocalError::Uuid(e) => PortError::Internal(e.to_string()),
            LocalError::Serialization(e) => PortError::Internal(e.to_string()),
            LocalError::Argon2(e) => PortError::Internal(e.to_string()),
        }
    }
}