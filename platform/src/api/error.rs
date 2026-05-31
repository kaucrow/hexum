use serde_json::json;
use crate::api::*;

pub enum ApiError {
    Validation(HashMap<String, Vec<String>>),
    Unauthorized(String),
    BadRequest(String),
    Conflict(String),
    NotFound(String),
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            Self::Validation(errors) => (StatusCode::UNPROCESSABLE_ENTITY, json!({ "errors": errors })), 
            Self::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, json!({ "error": msg })),
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, json!({ "error": msg })),
            Self::Conflict(msg) => (StatusCode::CONFLICT, json!({ "error": msg })),
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, json!({ "error": msg })),
            Self::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, json!({ "error": msg })),
        };

        (status, Json(body)).into_response()
    }
}