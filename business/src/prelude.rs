pub use std::{
    sync::Arc,
    collections::BTreeMap,
};
pub use tracing::{info, error};
pub use serde::{Serialize, Deserialize};
pub use anyhow::{Result, Context};

use std::path::PathBuf;
use serde::Deserializer;

/// Resolves crate assets path safely for both development and production.
pub fn get_crate_assets_path() -> PathBuf {
    if cfg!(debug_assertions) {
        // Development: Point straight to the crate's directory
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    } else {
        // Production: Look inside a namespaced folder relative to the executable
        std::env::current_dir()
            .expect("Failed to get current directory")
            .join("business")
    }
}

pub fn empty_is_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    // Parse the field as an optional string
    let opt: Option<String> = Option::deserialize(deserializer)?;

    // Filter out empty or whitespace-only strings
    Ok(opt.filter(|s| !s.trim().is_empty()))
}