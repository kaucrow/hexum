use crate::{
    prelude::*,
    api::*,
};

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BusinessHealthResponse {
    #[schema(example = "Business is healthy.")]
    pub message: String,
}