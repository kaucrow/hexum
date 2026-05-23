use axum::Json;

use crate::prelude::*;
use crate::api::ApiError;
use super::dtos::*;

#[utoipa::path(
    get,
    path = "/business-health",
    description = "Checks if the business logic is healthy.",
    responses(
        (status = 200, description = "Business is healthy"),
        (status = 500, description = "Internal Server Error")
    ),
    tags = ["Core"]
)]
pub async fn health() -> Result<Json<BusinessHealthResponse>, ApiError> {
    info!("Checking business health...");

    Ok(Json(BusinessHealthResponse {
        message: "Business is healthy.".to_string(),
    }))
}