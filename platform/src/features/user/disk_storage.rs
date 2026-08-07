use async_trait::async_trait;
use uuid::Uuid;

use crate::prelude::*;
use super::output::{FileStorage, FileStorageError};

/// Disk-based implementation of [`FileStorage`].
///
/// Files are written under `upload_dir / {user_id}.{ext}` so that every user
/// has at most one profile picture, and subsequent uploads overwrite the old one.
#[derive(Clone)]
pub struct DiskStorage {
    upload_dir: PathBuf,
}

impl DiskStorage {
    pub fn new(upload_dir: PathBuf) -> Self {
        Self { upload_dir }
    }

    /// Maps an image MIME type to a file extension.
    /// Returns `Err` for unsupported types instead of using a fallback.
    fn mime_to_extension(mime: &str) -> Result<&'static str, FileStorageError> {
        match mime {
            "image/jpeg" | "image/jpg" => Ok("jpg"),
            "image/png" => Ok("png"),
            "image/gif" => Ok("gif"),
            "image/webp" => Ok("webp"),
            "image/svg+xml" => Ok("svg"),
            "image/bmp" => Ok("bmp"),
            other => Err(FileStorageError::UnsupportedMimeType(other.to_string())),
        }
    }
}

#[async_trait]
impl FileStorage for DiskStorage {
    async fn save_image(
        &self,
        user_id: &Uuid,
        bytes: &[u8],
        content_type: &str,
    ) -> Result<String, FileStorageError> {
        let ext = Self::mime_to_extension(content_type)?;
        let filename = format!("{}.{}", user_id, ext);
        let filepath = self.upload_dir.join(&filename);

        if let Some(parent) = filepath.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| FileStorageError::Io(e.to_string()))?;
        }

        tokio::fs::write(&filepath, bytes)
            .await
            .map_err(|e| FileStorageError::Io(e.to_string()))?;

        info!("Profile picture saved to {:?}", filepath);

        Ok(format!("/uploads/{}", filename))
    }
}