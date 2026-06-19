pub(crate) mod routes;
mod error;
mod docs;

pub use docs::Docs;

#[allow(unused_imports)]
pub(crate) use platform::api::*;

pub(crate) use error::ApiError;

use axum::{Router, routing::get};
use crate::BusinessState;

pub fn router(state: BusinessState) -> Router {
    Router::new()
        .route("/business-health", get(crate::routes::health::health))
        .route("/games/search", get(crate::routes::games::search))
        .with_state(state)
}