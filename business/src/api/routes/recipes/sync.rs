use crate::{
    prelude::*,
    api::*,
    features::data_ingestion,
};

#[utoipa::path(
    get,
    path = "/recipe/sync",
    description = "Syncs the local recipes from the external API.",
    responses(
        (status = 202, description = "Began Recipes Sync"),
        (status = 500, description = "Internal Server Error")
    ),
    tags = ["Recipe"]
)]
pub async fn sync(
    State(data_ingestion_service): State<Arc<dyn data_ingestion::UseCase>>,
) -> Result<StatusCode, ApiError> {
    let service_clone = Arc::clone(&data_ingestion_service);

    tokio::spawn(async move {
        info!("Endpoint-triggered recipe sync started.");

        if let Err(e) = service_clone.sync_data().await {
            error!("Recipe sync failed: {:?}", e);
        } else {
            info!("Recipe sync completed successfully.");
        }
    });

    Ok(StatusCode::ACCEPTED)
}

impl From<data_ingestion::UseCaseError> for ApiError {
    fn from(e: data_ingestion::UseCaseError) -> Self {
        #[allow(unreachable_patterns)]
        match e {
            data_ingestion::UseCaseError::Internal(e) => {
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