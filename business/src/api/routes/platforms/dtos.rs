use uuid::Uuid;

use crate::{
    prelude::*,
    api::*,
};

#[derive(Serialize, ToSchema)]
pub struct PlatformResponse {
    pub id: Uuid,
    pub external_id: Option<i64>,
    pub name: String,
    pub generation: Option<u8>,
}

#[derive(Serialize, ToSchema)]
pub struct PlatformSyncResponse {
    /// Success confirmation message.
    #[schema(example = "Platforms synced successfully.")]
    pub message: String,
}
