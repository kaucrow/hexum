use crate::{
    prelude::*,
    api::*,
};
use utoipa::ToSchema;
use validator::Validate;

// ─── Submit Review ───

#[derive(Deserialize, ToSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct SubmitReviewRequest {
    /// Rating from 0 to 100.
    #[schema(example = 100)]
    #[validate(range(min = 0, max = 100))]
    pub rating: i32,

    /// Title of the review.
    #[schema(example = "Peak in the woods")]
    #[validate(length(min = 1, max = 255))]
    pub title: String,

    /// Content body of the review.
    #[schema(example = "I mean, what is there to say about it? It's just peak. I r8 8/8 gr8 game m8")]
    #[validate(length(min = 1))]
    pub content: String,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(example = json!({
    "id": "05639468-710b-44fe-9fc7-372514e95c37",
    "userId": "15639468-710b-44fe-9fc7-372514e95c37",
    "gameId": "25639468-710b-44fe-9fc7-372514e95c37",
    "reviewType": "critic",
    "rating": 100,
    "title": "Peak in the woods",
    "content": "I mean, what is there to say about it? It's just peak. I r8 8/8 gr8 game m8",
    "createdAt": "2025-06-29T20:58:27.379Z",
    "updatedAt": "2025-06-29T20:58:27.379Z"
}))]
pub struct ReviewResponse {
    pub id: String,
    pub user_id: String,
    pub game_id: String,
    pub review_type: String,
    pub rating: i32,
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

// ─── Get Game Reviews ───

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetGameReviewsResponse {
    pub reviews: Vec<ReviewResponse>,
}

// ─── Path Params ───

#[derive(Deserialize, IntoParams, Validate)]
pub struct GameIdPath {
    /// The game's internal UUID.
    pub id: String,
}