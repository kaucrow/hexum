use crate::{
    prelude::*,
    api::*,
};

#[derive(Serialize, ToSchema)]
pub struct BusinessHealthResponse {
    #[schema(example = "Business is healthy.")]
    pub message: String,
}