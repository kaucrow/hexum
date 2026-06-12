use serde_json::json;
use crate::{
    api::*,
    features::ratelimit,
};

pub enum ApiError {
    Validation(HashMap<String, Vec<String>>),
    Unauthorized(String),
    BadRequest(String),
    Conflict(String),
    NotFound(String),
    TooManyRequests(String, Option<u64>),
    Internal,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            Self::Validation(errors) => (StatusCode::UNPROCESSABLE_ENTITY, json!({ "errors": errors })),
            Self::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, json!({ "error": msg })),
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, json!({ "error": msg })),
            Self::Conflict(msg) => (StatusCode::CONFLICT, json!({ "error": msg })),
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, json!({ "error": msg })),
            Self::TooManyRequests(msg, retry_after) => {
                let mut response = (StatusCode::TOO_MANY_REQUESTS, Json(json!({ "error": msg }))).into_response();
                if let Some(seconds) = retry_after {
                    response.headers_mut().insert(
                        axum::http::HeaderName::from_static("retry-after"),
                        axum::http::HeaderValue::from_str(&seconds.to_string()).unwrap(),
                    );
                }
                return response;
            }
            Self::Internal => (StatusCode::INTERNAL_SERVER_ERROR, json!({ "error": "An internal error occurred" })),
        };

        (status, Json(body)).into_response()
    }
}


impl From<ratelimit::UseCaseError> for ApiError {
    fn from(e: ratelimit::UseCaseError) -> Self {
        match e {
            ratelimit::UseCaseError::TooManyRequests { retry_after_secs } => {
                ApiError::TooManyRequests(
                    "Too many login attempts from this IP. Please slow down.".to_string(),
                    Some(retry_after_secs),
                )
            }
            ratelimit::UseCaseError::LockedOut { retry_after_secs } => {
                ApiError::TooManyRequests(
                    "Account temporarily locked.".to_string(),
                    Some(retry_after_secs),
                )
            }
            ratelimit::UseCaseError::Internal(_) => ApiError::Internal,
        }
    }
}