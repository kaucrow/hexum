use std::sync::Arc;
use utoipa::OpenApi;
use axum::{
    http::{header, StatusCode},
    response::Response,
    extract::Request,
};
use tower_http::validate_request::ValidateRequestHeaderLayer;

use platform::Config;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "NativEat API",
        description = "Hexum-based API for the NativEat mobile app",
    )
)]
pub struct MasterDocs;

pub fn get_auth_layer(config: Arc<Config>) -> ValidateRequestHeaderLayer<
    impl FnMut(&mut Request) -> Result<(), Response> + Clone
>
{
    ValidateRequestHeaderLayer::custom({
        move |request: &mut axum::extract::Request| {
            // Extract the authorization header
            let auth_header = request
                .headers()
                .get(header::AUTHORIZATION)
                .and_then(|value| value.to_str().ok());

            if let Some(auth_str) = auth_header {
                // Check if it's a basic auth header
                if auth_str.starts_with("Basic ") {
                    let encoded = &auth_str[6..];
                    // Decode the base64 credentials
                    if let Ok(decoded_bytes) = base64::Engine::decode(
                        &base64::prelude::BASE64_STANDARD, 
                        encoded
                    ) {
                        if let Ok(credentials) = String::from_utf8(decoded_bytes) {
                            // Split into username and password
                            if let Some((user, password)) = credentials.split_once(':') {
                                // If the a username with this password is in the config, grant access
                                if let Some(expected_password) = config.api.docs_users.get(user) {
                                    if expected_password == password {
                                        return Ok(());
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // If anything fails or doesn't match, return a 401 Unauthorized
            // and include the WWW-Authenticate header so browsers prompt for a login.
            let mut response = axum::response::Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(axum::body::Body::empty())
                .unwrap();

            response.headers_mut().insert(
                header::WWW_AUTHENTICATE,
                header::HeaderValue::from_static("Basic realm=\"Secure API Docs\""),
            );

            Err(response)
        }
    })
}