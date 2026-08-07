pub use std::{
    sync::Arc,
    path::PathBuf,
};
pub use thiserror::Error;
pub use async_trait::async_trait;
pub use uuid::Uuid;
pub use tracing::{info, error};
pub use serde::{Serialize, Deserialize};
pub use anyhow::{Result, Context};

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