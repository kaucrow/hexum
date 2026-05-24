use crate::{
    prelude::*,
    api::*,
    features::base,
};
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
pub async fn health(
    State(base_service): State<Arc<dyn base::UseCase>>,
) -> Result<Json<BusinessHealthResponse>, ApiError> {
    info!("Checking business health...");

    base_service.check_db_health().await?;

    Ok(Json(BusinessHealthResponse {
        message: "Business is healthy.".to_string(),
    }))
}

impl From<base::UseCaseError> for ApiError {
    fn from(e: base::UseCaseError) -> Self {
        #[allow(unreachable_patterns)]
        match e {
            base::UseCaseError::Internal(e) => {
                error!("An internal error occurred: {e}");
                ApiError::Internal("An internal error occurred".to_string())
            }
            _ => {
                error!("Unexpected domain error: {e}");
                ApiError::Internal("An internal error occurred".to_string())
            }
        }
    }
}