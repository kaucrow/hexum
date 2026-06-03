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

use crate::features::user;
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

    fn do_hash_password(&self, password: &user::Password) -> Result<String, LocalError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hash = argon2.hash_password(password.as_str().as_bytes(), &salt)?.to_string();

        Ok(hash)
    }

    fn do_verify_access_token(&self, token: &str) -> Result<Uuid, LocalError> {
        let validation_rules = ClaimsValidationRules::new();
        let untrusted_token = UntrustedToken::<Local, V4>::try_from(token)
            .map_err(|_| LocalError::TokenVerificationFailed)?;
        let trusted_token = local::decrypt(&self.sk, &untrusted_token, &validation_rules, None, None)
            .map_err(|_| LocalError::TokenVerificationFailed)?;

        let user_id: String = trusted_token
            .payload_claims()
            .and_then(|claims| claims.get_claim("user_id"))
            .and_then(|json_value| json_value.as_str())
            .map(|s| s.to_string())
            .ok_or(LocalError::TokenVerificationFailed)?;

        let user_id_uuid = Uuid::try_parse(&user_id)?;

        Ok(user_id_uuid)
    }

    fn do_generate_access_token(&self, user_id: &Uuid) -> Result<String, LocalError> {
        let mut claims = Claims::new()?;
        claims.add_additional("user_id", user_id.to_string())?;

        // Expiration will be 24 hours from current time
        let expiration = Utc::now()
            .checked_add_signed(Duration::hours(24))
            .expect("Timestamp should not overflow");

        // Format the expiration to RFC3339 and set it
        claims.expiration(&expiration.to_rfc3339())?;

        // Encrypt the claims
        let token = local::encrypt(&self.sk, &claims, None, None)?;

        Ok(token)
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
        Ok(self.do_hash_password(password)?)
    }

    // Verify a PASETO v4 access token & return the user_id
    fn verify_access_token(&self, token: &str) -> Result<Uuid, PortError> {
        Ok(self.do_verify_access_token(token)?)
    }

    // Generate a PASETO v4 access token
    fn generate_access_token(&self, user_id: &Uuid) -> Result<String, PortError> {
        Ok(self.do_generate_access_token(user_id)?)
    }

    // Generate a 64-characters long refresh token
    fn generate_refresh_token(&self) -> String {
        Alphanumeric.sample_string(&mut rand::rng(), 64)
    }

    // Generate a 6-digit verification code (zero-padded, e.g. "042739")
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
    Argon2(#[from] argon2::password_hash::Error),
}

impl From<LocalError> for PortError {
    fn from(e: LocalError) -> Self {
        match e {
            LocalError::TokenVerificationFailed => PortError::TokenVerificationFailed,
            LocalError::Paseto(e) => PortError::Internal(e.to_string()),
            LocalError::Uuid(e) => PortError::Internal(e.to_string()),
            LocalError::Argon2(e) => PortError::Internal(e.to_string()),
        }
    }
}