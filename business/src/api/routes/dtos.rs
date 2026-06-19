use utoipa::ToSchema;

use crate::prelude::*;

#[derive(Serialize, ToSchema)]
pub struct BusinessHealthResponse {
    #[schema(example = "Business is healthy.")]
    pub message: String,
}