use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::Serialize;

// ─── Domain types ───

#[derive(Debug, Clone, Serialize)]
pub struct UserSummary {
    pub id: Uuid,
    pub username: String,
    pub profile_picture_url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FriendRequestStatus {
    Pending,
    Accepted,
    Rejected,
}

impl FriendRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            FriendRequestStatus::Pending => "pending",
            FriendRequestStatus::Accepted => "accepted",
            FriendRequestStatus::Rejected => "rejected",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FriendRequest {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub sender_username: String,
    pub receiver_id: Uuid,
    pub receiver_username: String,
    pub status: FriendRequestStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── Output port ───

#[async_trait]
pub trait Port: Send + Sync + 'static {
    async fn list_strangers(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<UserSummary>, PortError>;

    async fn create_request(
        &self,
        id: Uuid,
        sender_id: Uuid,
        receiver_id: Uuid,
    ) -> Result<FriendRequest, PortError>;

    async fn get_sent_requests(
        &self,
        sender_id: Uuid,
    ) -> Result<Vec<FriendRequest>, PortError>;

    async fn get_received_requests(
        &self,
        receiver_id: Uuid,
    ) -> Result<Vec<FriendRequest>, PortError>;

    async fn accept_request(
        &self,
        request_id: Uuid,
        receiver_id: Uuid,
    ) -> Result<Option<FriendRequest>, PortError>;

    async fn reject_request(
        &self,
        request_id: Uuid,
        receiver_id: Uuid,
    ) -> Result<Option<FriendRequest>, PortError>;

    async fn get_friends(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<UserSummary>, PortError>;
}

// ─── Errors ───

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ConflictError {
    #[error("A pending friend request already exists between these users.")]
    PendingRequestExists,
}

#[derive(Error, Debug)]
pub enum PortError {
    #[error(transparent)]
    Conflict(#[from] ConflictError),

    #[error("Friends repository: {0}")]
    Internal(String),
}