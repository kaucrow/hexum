use async_trait::async_trait;
use thiserror::Error;

use crate::prelude::*;

#[async_trait]
pub trait Port: Send + Sync + 'static {
    /// Performs a request on the Videogame API.
    /// Handles request -> 401 Unauthorized -> re-authentication -> retry.
    ///
    /// The caller never needs to know about token expiry.
    async fn request<T: DeserializeOwned + Send + 'static>(
        &self,
        endpoint: &str,
        body: &str,
    ) -> Result<T, PortError>;
}

#[derive(Clone)]
pub struct VideogameApiAuthData {
    pub url: String,
    pub client_id: String,
    pub client_secret: String,
}

pub type VideogameApiToken = String;

#[derive(Error, Debug)]
pub enum PortError {
    #[error("External error in Videogame API auth: {0}")]
    Http(String),

    #[error("Videogame API Auth error: {0}")]
    Internal(String),
}