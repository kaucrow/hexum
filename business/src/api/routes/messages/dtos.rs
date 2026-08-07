use crate::{
    prelude::*,
    api::*,
};

// ─── DTOs ───

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MessageResponse {
    #[schema(format = "uuid")]
    pub id: String,

    #[schema(format = "uuid")]
    pub sender_id: String,

    #[schema(format = "uuid")]
    pub receiver_id: String,

    pub content: String,

    pub created_at: String,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(example = json!({
    "messages": [
        {"id": "...", "senderId": "...", "receiverId": "...", "content": "Hello!", "createdAt": "2024-01-01T00:00:00Z"}
    ],
    "limit": 50,
    "offset": 0
}))]
pub struct ConversationResponse {
    pub messages: Vec<MessageResponse>,
    pub limit: i64,
    pub offset: i64,
}

// ─── Pagination ───

pub use super::super::friends::dtos::PaginationQuery;