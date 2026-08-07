use crate::{
    prelude::*,
    api::*,
};

// ─── Shared DTO ───

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LastMessageResponse {
    #[schema(format = "uuid")]
    pub id: String,

    #[schema(format = "uuid")]
    pub sender_id: String,

    /// Either "text" or "image".
    #[schema(example = "text")]
    pub r#type: String,

    /// For text: the message content. For image: the image URL.
    pub content: String,

    pub created_at: String,
}

// ─── Pagination ───

#[derive(Deserialize, ToSchema, Validate, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct PaginationQuery {
    /// Maximum number of users to return (1-100).
    #[schema(example = 20, minimum = 1, maximum = 100)]
    #[validate(range(min = 1, max = 100))]
    pub limit: Option<i64>,

    /// Number of users to skip before starting to collect the result set.
    #[schema(example = 0, minimum = 0)]
    #[validate(range(min = 0))]
    pub offset: Option<i64>,
}

impl PaginationQuery {
    pub fn limit(&self) -> i64 {
        self.limit.unwrap_or(20)
    }

    pub fn offset(&self) -> i64 {
        self.offset.unwrap_or(0)
    }
}

// ─── User listing ───

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserSummaryResponse {
    /// The user's ID (UUID).
    #[schema(format = "uuid")]
    pub id: String,

    pub username: String,

    /// URL to the user's profile picture, if set.
    pub profile_picture_url: Option<String>,

    /// The last message exchanged with this user, if any.
        pub last_message: Option<LastMessageResponse>,
    }
    
    impl From<crate::features::friends::LastMessage> for LastMessageResponse {
        fn from(lm: crate::features::friends::LastMessage) -> Self {
            Self {
                id: lm.id.to_string(),
                sender_id: lm.sender_id.to_string(),
                r#type: lm.message_type,
                content: lm.content,
                created_at: lm.created_at.to_rfc3339(),
            }
        }
    }

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(example = json!({
    "users": [
        {"id": "05639468-710b-44fe-9fc7-372514e95c37", "username": "johndoe", "profilePictureUrl": "/uploads/abc123.png"}
    ],
    "limit": 20,
    "offset": 0
}))]
pub struct UserListResponse {
    pub users: Vec<UserSummaryResponse>,
    pub limit: i64,
    pub offset: i64,
}

// ─── Friend request ───

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(example = json!({
    "id": "05639468-710b-44fe-9fc7-372514e95c37",
    "senderId": "abc123",
    "senderUsername": "alice",
    "receiverId": "def456",
    "receiverUsername": "bob",
    "status": "pending",
    "createdAt": "2024-01-01T00:00:00Z",
    "updatedAt": "2024-01-01T00:00:00Z"
}))]
pub struct FriendRequestResponse {
    #[schema(format = "uuid")]
    pub id: String,

    #[schema(format = "uuid")]
    pub sender_id: String,

    pub sender_username: String,

    #[schema(format = "uuid")]
    pub receiver_id: String,

    pub receiver_username: String,

    /// One of: `pending`, `accepted`, `rejected`.
    pub status: String,

    pub created_at: String,

    pub updated_at: String,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SendFriendRequestResponse {
    #[schema(example = "Friend request sent successfully.")]
    pub message: String,

    pub request: FriendRequestResponse,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FriendRequestListResponse {
    pub requests: Vec<FriendRequestResponse>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AcceptRejectResponse {
    #[schema(example = "Friend request accepted.")]
    pub message: String,

    pub request: FriendRequestResponse,
}

// ─── Friends list ───

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(example = json!({
    "friends": [
        {"id": "05639468-710b-44fe-9fc7-372514e95c37", "username": "johndoe", "profilePictureUrl": "/uploads/abc123.png"}
    ]
}))]
pub struct FriendListResponse {
    pub friends: Vec<UserSummaryResponse>,
}