use crate::{
    prelude::*,
    api::*,
};
use utoipa::ToSchema;

// ─── Apply for Critic ───

#[derive(Serialize, ToSchema)]
pub struct ApplyForCriticResponse {
    #[schema(example = "Your application for the Critic role has been submitted.")]
    pub message: String,
    pub application_id: String,
}

// ─── List Applications ───

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(example = json!({
    "id": "05639468-710b-44fe-9fc7-372514e95c37",
    "userId": "15639468-710b-44fe-9fc7-372514e95c37",
    "username": "johndoe",
    "email": "johndoe@gmail.com",
    "status": "pending",
    "appliedAt": "2025-06-29T20:58:27.379Z"
}))]
pub struct CriticApplicationResponse {
    pub id: String,
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub status: String,
    pub applied_at: String,
}

// ─── Approve / Reject Application ───

#[derive(Deserialize, ToSchema, Validate)]
pub struct ApplicationPathParams {
    #[validate(length(min = 1))]
    pub id: String,
}

#[derive(Serialize, ToSchema)]
pub struct ApproveApplicationResponse {
    #[schema(example = "Application approved. User has been granted the Critic role.")]
    pub message: String,
}

#[derive(Serialize, ToSchema)]
pub struct RejectApplicationResponse {
    #[schema(example = "Application rejected.")]
    pub message: String,
}