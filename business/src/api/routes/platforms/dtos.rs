use uuid::Uuid;

use crate::{
    prelude::*,
    api::*,
};

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlatformResponse {
    pub id: Uuid,
    pub external_id: Option<i64>,
    pub name: String,
    pub generation: Option<u8>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlatformSyncResponse {
    /// Success confirmation message.
    #[schema(example = "Platforms synced successfully.")]
    pub message: String,
}
