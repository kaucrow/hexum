use crate::prelude::*;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct BusinessHealthResponse {
    #[schema(example = "Business is healthy.")]
    pub message: String,
}