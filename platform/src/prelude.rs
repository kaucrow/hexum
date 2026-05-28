pub use std::{
    sync::Arc,
    collections::HashMap,
};
pub use tracing::{info, warn, error};
pub use serde::{Serialize, Deserialize};
pub use anyhow::{Result, Context};

use std::path::PathBuf;

/// Finds the base path for config files.
/// In development (cargo run), it resolves to the workspace root.
/// In production (direct execution), it uses the folder containing the executable.
pub fn get_root_path() -> PathBuf {
    // Are we running via `cargo run`?
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let mut path = PathBuf::from(manifest_dir);

        // If the path ends in a known workspace member folder (like 'api' or 'platform'),
        // step up one level to get to the true workspace root.
        if path.ends_with("api")
            || path.ends_with("platform")
            || path.ends_with("business")
            || path.ends_with("tests")
        {
            path.pop();
        }

        return path;
    }

    // Otherwise, we are in production. Use the directory where the app is executed.
    std::env::current_dir().expect("Failed to determine current working directory.")
}

/// Resolves crate assets path safely for both development and production.
pub fn get_crate_assets_path() -> PathBuf {
    if cfg!(debug_assertions) {
        // Development: Point straight to the crate's directory
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    } else {
        // Production: Look inside a namespaced folder relative to the executable
        std::env::current_dir()
            .expect("Failed to get current directory")
            .join("platform")
    }
}