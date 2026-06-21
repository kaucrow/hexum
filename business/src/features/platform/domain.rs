use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct Platform {
    pub id: Uuid,
    pub external_id: Option<u64>,
    pub name: String,
    pub generation: Option<u8>,
}