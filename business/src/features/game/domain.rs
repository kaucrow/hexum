use crate::prelude::*;

pub struct Game {
    pub id: Uuid,
    pub external_id: Option<u64>,
    pub name: String,
    pub platforms: Vec<Platform>,
}

pub struct Platform {
    pub id: Uuid,
    pub external_id: Option<u64>,
    pub name: String,
    pub generation: Option<u8>,
}