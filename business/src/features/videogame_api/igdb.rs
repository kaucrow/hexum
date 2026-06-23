use reqwest::{Client as HttpClient, StatusCode};

use crate::prelude::*;
use super::*;

#[derive(Clone)]
pub struct IgdbClient {
    http_client: HttpClient,
    auth_data: VideogameApiAuthData,
    access_token: Arc<RwLock<Option<VideogameApiToken>>>,
}

impl IgdbClient {
    pub fn new(http_client: HttpClient, auth_data: VideogameApiAuthData) -> Self {
        Self {
            http_client,
            auth_data,
            access_token: Arc::new(RwLock::new(None)),
        }
    }
}

impl IgdbClient {
    async fn get_token(&self) -> Result<VideogameApiToken, LocalError> {
        let res: Result<_, reqwest::Error> = async {
            let response: TwitchAuthResponse = self.http_client
                .post(&self.auth_data.url)
                .query(&[
                    ("client_id", &self.auth_data.client_id),
                    ("client_secret", &self.auth_data.client_secret),
                    ("grant_type", &"client_credentials".to_string()),
                ])
                .send()
                .await?
                .error_for_status()?
                .json::<TwitchAuthResponse>()
                .await?;

            Ok(response.access_token)
        }.await;

        res.map_err(Into::into)
    }

    /// Ensures a cached token exists, fetching one if needed.
    async fn ensure_token(&self) -> Result<VideogameApiToken, PortError> {
        // Scope the read guard so it's dropped before any `.await`
        let cached = {
            self.access_token.read().expect("IgdbClient access_token lock poisoned").clone()
        };
        if let Some(token) = cached {
            return Ok(token);
        }

        let token = self.get_token().await?;
        {
            *self.access_token.write().expect("IGDB access_token lock poisoned") = Some(token.clone());
        }
        Ok(token)
    }

    /// Performs a single IGDB HTTP request with the given token.
    async fn try_request<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &str,
        token: &str,
    ) -> Result<T, LocalError> {
        let response = self
            .http_client
            .post(endpoint)
            .header("Client-ID", &self.auth_data.client_id)
            .header("Authorization", format!("Bearer {}", token))
            .body(body.to_owned())
            .send()
            .await?;

        if response.status() == StatusCode::UNAUTHORIZED {
            return Err(LocalError::Unauthorized);
        }

        let result = response.error_for_status()?.json::<T>().await?;

        Ok(result)
    }
}

#[async_trait]
impl Port for IgdbClient {
    /// Sends a POST request to the given IGDB endpoint with the given body.
    /// On 401, re-authenticates once and retries.
    async fn request<T: DeserializeOwned + Send + 'static>(
        &self,
        endpoint: &str,
        body: &str,
    ) -> Result<T, PortError> {
        // Ensure we have a token
        let token = self.ensure_token().await?;

        // First attempt
        match self.try_request::<T>(endpoint, body, &token).await {
            Ok(result) => Ok(result),
            Err(LocalError::Unauthorized) => {
                // Token expired — re-authenticate and retry once
                let new_token = self.get_token().await?;
                {
                    *self.access_token.write().expect("IgdbClient access_token lock poisoned") = Some(new_token.clone());
                }

                self.try_request::<T>(endpoint, body, &new_token)
                    .await
                    .map_err(Into::into)
            }
            Err(e) => Err(e.into()),
        }
    }
}

#[async_trait]
impl Port for Arc<IgdbClient> {
    async fn request<T: DeserializeOwned + Send + 'static>(
        &self,
        endpoint: &str,
        body: &str,
    ) -> Result<T, PortError> {
        (**self).request(endpoint, body).await
    }
}

#[derive(Error, Debug)]
pub enum LocalError {
    #[error("IGDB Unauthorized")]
    Unauthorized,

    #[error(transparent)]
    Http(#[from] reqwest::Error),
}

impl From<LocalError> for PortError {
    fn from(e: LocalError) -> Self {
        match e {
            LocalError::Unauthorized => PortError::Internal("Failed to authenticate on IGDB.".to_string()),
            LocalError::Http(e) => PortError::Http(e.to_string()),
        }
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct TwitchAuthResponse {
    access_token: String,
    expires_in: u64,
    token_type: String,
}