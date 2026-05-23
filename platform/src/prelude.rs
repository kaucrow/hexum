pub use tracing::{debug, info, warn, error};
pub use serde::{Serialize, Deserialize};
pub use anyhow::Context;

use std::path::PathBuf;

use strum::Display;

#[derive(Display)]
#[strum(serialize_all = "lowercase")]
pub enum Environment {
    Development,
    Production,
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "development" => Ok(Self::Development),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either `development` or `production`.",
                other
            )),
        }
    }
}

/// Finds the base path for config files.
/// In development (cargo run), it resolves to the workspace root.
/// In production (direct execution), it uses the folder containing the executable.
pub fn get_root_path() -> PathBuf {
    // Are we running via `cargo run`?
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let mut path = PathBuf::from(manifest_dir);

        // If the path ends in a known workspace member folder (like 'api' or 'platform'),
        // step up one level to get to the true workspace root.
        if path.ends_with("api") || path.ends_with("platform") || path.ends_with("business") {
            path.pop();
        }

        return path;
    }

    // Otherwise, we are in production. Use the directory where the app is executed.
    std::env::current_dir().expect("Failed to determine current working directory.")
}

/// Resolves crate assets path safely for both development and production.
/// `crate_subfolder` is the name of the folder inside production (e.g., "platform" or "business").
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