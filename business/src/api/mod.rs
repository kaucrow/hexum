pub(crate) mod routes;
mod error;
mod docs;

pub use docs::Docs;

#[allow(unused_imports)]
pub(crate) use axum::{
    Json,
    extract::{State, Query, Path},
    response::{Html, Response, IntoResponse},
    http::StatusCode,
};

#[allow(unused_imports)]
pub(crate) use utoipa::{ToSchema, IntoParams};
pub(crate) use validator::Validate;

pub(crate) use error::ApiError;
pub(crate) use platform::api::{
    AuthenticatedUser,
    ValidatedQuery,
};

use axum::{Router, routing::get, routing::post};
use crate::BusinessState;

pub fn router(state: BusinessState) -> Router {
    Router::new()
        .route("/business-health", get(crate::routes::health::health))
        .route("/users", get(crate::routes::friends::list_users))
        .route("/friends", get(crate::routes::friends::list_friends))
        .route("/users/{user_id}/friend-request", post(crate::routes::friends::send_friend_request))
        .route("/friend-requests/sent", get(crate::routes::friends::get_sent_requests))
        .route("/friend-requests/received", get(crate::routes::friends::get_received_requests))
        .route("/friend-requests/{request_id}/accept", post(crate::routes::friends::accept_friend_request))
        .route("/friend-requests/{request_id}/reject", post(crate::routes::friends::reject_friend_request))
        .with_state(state)
}