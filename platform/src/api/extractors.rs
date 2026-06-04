use std::marker::PhantomData;
use std::ops::Deref;

use crate::{
    prelude::*,
    api::*,
    features::{user, auth},
};

/// Generic extractor that requires an authenticated user and works with
/// any state type `T` that implements `FromRef<Arc<dyn auth::UseCase>>`.
/// Both `PlatformState` and `BusinessState` satisfy this bound via their
/// `#[derive(FromRef)]` macros.
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub roles: Vec<user::Role>,
    pub provider: user::AuthProvider,
}

impl AuthenticatedUser {
    /// Check if the user has any of the given roles.
    pub fn has_any_role(&self, allowed_roles: &[user::Role]) -> bool {
        self.roles.iter().any(|user_role| allowed_roles.contains(user_role))
    }
}

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

        let session = auth_service
            .verify_session(access_token)
            .await
            .map_err(|e| {
                warn!("Session verification failed: {e}");
                StatusCode::UNAUTHORIZED
            })?;

        let user_id = session.user_id;
        let roles = session.roles;
        let provider = session.provider;

        info!(
            "Session verification successful for user with ID '{}', via '{}'",
            user_id,
            provider,
        );

        Ok(Self { user_id, roles, provider })
    }
}

pub mod role {
    use crate::features::user;

    /// The trait that binds a type to a specific `user::Role` variant.
    pub trait RoleMarker: Send + Sync + 'static {
        const ROLE: user::Role;
    }

    pub struct Admin;
    impl RoleMarker for Admin { const ROLE: user::Role = user::Role::Admin; }

    pub struct Manager;
    impl RoleMarker for Manager { const ROLE: user::Role = user::Role::Manager; }

    pub struct BasicUser;
    impl RoleMarker for BasicUser { const ROLE: user::Role = user::Role::BasicUser; }
}

/// Generic extractor that requires a specific role defined by a `RoleMarker`.
pub struct RequireRole<M: role::RoleMarker>(
    pub AuthenticatedUser,
    pub PhantomData<M>,
);

impl<T, M> FromRequestParts<T> for RequireRole<M>
where
    T: Send + Sync,
    Arc<dyn auth::UseCase>: FromRef<T>,
    M: role::RoleMarker,
{
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &T,
    ) -> Result<Self, Self::Rejection> {
        let auth = AuthenticatedUser::from_request_parts(parts, state).await?;

        if !auth.roles.contains(&M::ROLE) {
            return Err(StatusCode::FORBIDDEN);
        }

        Ok(Self(auth, PhantomData))
    }
}

impl<M: role::RoleMarker> Deref for RequireRole<M> {
    type Target = AuthenticatedUser;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}