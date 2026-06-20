use crate::{
    prelude::*,
    api::*,
    features::platform,
};
use super::dtos::*;

#[utoipa::path(
    get,
    path = "/platforms",
    description = "Returns all platforms stored in the internal database.",
    responses(
        (status = 200, description = "List of platforms", body = Vec<PlatformResponse>),
        (status = 500, description = "Internal Server Error")
    ),
    tags = ["Platforms"]
)]
pub async fn get_platforms(
    State(platform_service): State<Arc<dyn platform::UseCase>>,
) -> Result<Json<Vec<PlatformResponse>>, ApiError> {
    info!("Got request to list all platforms.");

    let platforms = platform_service
        .get_platforms()
        .await?;

    let dtos: Vec<PlatformResponse> = platforms
        .into_iter()
        .map(|p| PlatformResponse {
            id: p.id,
            external_id: p.external_id.map(|eid| eid as i64),
            name: p.name,
            generation: p.generation,
        })
        .collect();

    Ok(Json(dtos))
}

impl From<platform::UseCaseError> for ApiError {
    fn from(e: platform::UseCaseError) -> Self {
        match e {
            platform::UseCaseError::VideogameApi(msg) => {
                error!("Videogame API error in platform: {msg}");
                ApiError::Internal("An internal error occurred".to_string())
            }
            platform::UseCaseError::Internal(msg) => {
                error!("An internal error occurred in platform: {msg}");
                ApiError::Internal("An internal error occurred".to_string())
            }
        }
    }
}