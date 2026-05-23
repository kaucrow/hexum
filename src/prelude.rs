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
/// In development (cargo run), it uses the project root.
/// In production (direct execution), it uses the folder containing the executable.
pub fn get_base_path() -> PathBuf {
    // Are we running via `cargo run`? If so, use the project root
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        return PathBuf::from(manifest_dir);
    }

    // Otherwise, we are in production. Use the directory where the app is being executed from.
    std::env::current_dir().expect("Failed to determine current working directory.")
}