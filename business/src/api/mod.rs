pub(crate) mod routes;
mod error;
mod docs;

pub use docs::Docs;

#[allow(unused_imports)]
pub(crate) use platform::api::*;

pub(crate) use error::ApiError;

use axum::{Router, routing::{get, post}};
use crate::BusinessState;

pub fn router(state: BusinessState) -> Router {
    Router::new()
        .route("/business-health", get(crate::routes::health::health))
        .route("/game", get(crate::routes::games::get_game))
        .route("/game/search", get(crate::routes::games::search))
        .route("/game/popular", get(crate::routes::games::popular_games))
        .route("/platforms", get(crate::routes::platforms::get_platforms))
        .route("/platforms/sync", post(crate::routes::platforms::sync_platforms))
        .with_state(state)
}