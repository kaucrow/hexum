use crate::{
    prelude::*,
    api::*,
    features::{user, auth},
};

#[allow(dead_code)]
pub struct AuthenticatedUser(pub user::User);

/// Generic extractor that works with any state type `T` that implements
/// `FromRef<Arc<dyn auth::UseCase>>`. Both `PlatformState` and `BusinessState`
/// satisfy this bound via their `#[derive(FromRef)]` macros.
impl<T> FromRequestParts<T> for AuthenticatedUser
where
    T: Send + Sync,
    Arc<dyn auth::UseCase>: axum::extract::FromRef<T>,
{
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &T,
    ) -> Result<Self, Self::Rejection> {
        info!("Session verification request received");

        // Pull the auth service from the generic state via FromRef
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