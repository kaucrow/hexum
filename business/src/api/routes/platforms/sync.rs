use crate::{
    prelude::*,
    api::*,
    features::platform,
};
use super::dtos::*;

#[utoipa::path(
    post,
    path = "/platforms/sync",
    description = "Triggers a sync of platforms from the external repository into the internal database.",
    responses(
        (status = 202, description = "Began platforms sync", body = PlatformSyncResponse),
        (status = 500, description = "Internal Server Error")
    ),
    tags = ["Platforms"]
)]
pub async fn sync_platforms(
    State(platform_service): State<Arc<dyn platform::UseCase>>,
) -> Result<StatusCode, ApiError> {
    let service_clone = Arc::clone(&platform_service);

    tokio::spawn(async move {
        info!("Endpoint-triggered recipe sync started.");

        if let Err(e) = service_clone.sync_db_platforms().await {
            error!("Platforms sync failed: {:?}", e);
        } else {
            info!("Platforms sync completed successfully.");
        }
    });

    Ok(StatusCode::ACCEPTED)
}