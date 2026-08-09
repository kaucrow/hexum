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

use tower_http::services::ServeDir;
use axum::{Router, routing::get, routing::post};
use crate::BusinessState;

pub fn router(state: BusinessState, upload_dir: String) -> Router {
    Router::new()
        .route("/business-health", get(crate::routes::health::health))
        // Friends
        .route("/users", get(crate::routes::friends::list_users))
        .route("/users/{user_id}/friend-request", post(crate::routes::friends::send_friend_request))
        .route("/friend-requests/sent", get(crate::routes::friends::get_sent_requests))
        .route("/friend-requests/received", get(crate::routes::friends::get_received_requests))
        .route("/friend-requests/{request_id}/accept", post(crate::routes::friends::accept_friend_request))
        .route("/friend-requests/{request_id}/reject", post(crate::routes::friends::reject_friend_request))
        .route("/friends", get(crate::routes::friends::get_friends))
        // Messages
        .route("/messages/{friend_id}", get(crate::routes::messages::get_conversation))
        .route("/ws/friends/{friend_id}", get(crate::routes::messages::ws_handler))
        // Uploads
        .nest_service("/uploads", ServeDir::new(&upload_dir))
        .with_state(state)
}