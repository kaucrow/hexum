use std::sync::Arc;

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use axum_extra::extract::cookie::CookieJar;

use crate::{
    prelude::*,
    PlatformState,
    features::{user, auth},
};

#[allow(dead_code)]
pub struct AuthenticatedUser(pub user::User);

impl FromRequestParts<PlatformState> for AuthenticatedUser {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &PlatformState,
    ) -> Result<Self, Self::Rejection> {
        info!("Session verification request received");

        // Pull dependencies from PlatformState
        let auth_service: Arc<dyn auth::UseCase> = axum::extract::FromRef::from_ref(state);

        // Grab the CookieJar from the incoming headers
        let jar = CookieJar::from_headers(&parts.headers);

        // Extract the "access_token" cookie
        let access_token = jar.get("access_token")
            .map(|cookie| cookie.value())
            .ok_or(StatusCode::UNAUTHORIZED)?;

        let user = auth_service
            .verify_user(access_token)
            .await
            .map_err(|e| {
                warn!("Session verification failed: {e}");
                StatusCode::UNAUTHORIZED
            })?;

        info!("Session verification successful for user `{}`", user.username.as_str());

        Ok(Self(user))
    }
}