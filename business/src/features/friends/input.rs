use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

use super::{UserSummary, FriendRequest};

#[async_trait]
pub trait UseCase: Send + Sync + 'static {
    /// List users who are not the current user, not already friends,
    /// and have no pending friend requests (either direction) with the current user.
    async fn list_users(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<UserSummary>, UseCaseError>;

    /// Send a friend request to another user.
    async fn send_friend_request(
        &self,
        sender_id: Uuid,
        receiver_id: Uuid,
    ) -> Result<FriendRequest, UseCaseError>;

    /// Get pending friend requests sent by the user.
    async fn get_sent_requests(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<FriendRequest>, UseCaseError>;

    /// Get pending friend requests received by the user.
    async fn get_received_requests(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<FriendRequest>, UseCaseError>;

    /// Accept a pending friend request. Only the receiver can accept.
    async fn accept_friend_request(
        &self,
        request_id: Uuid,
        current_user_id: Uuid,
    ) -> Result<FriendRequest, UseCaseError>;

    /// Reject a pending friend request. Only the receiver can reject.
    async fn reject_friend_request(
        &self,
        request_id: Uuid,
        current_user_id: Uuid,
    ) -> Result<FriendRequest, UseCaseError>;

    /// List all accepted friends of the user.
    async fn get_friends(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<UserSummary>, UseCaseError>;
}

// ─── Errors ───

#[derive(Error, Debug)]
pub enum UseCaseError {
    #[error("You cannot send a friend request to yourself.")]
    CannotFriendYourself,

    #[error("A pending friend request already exists between these users.")]
    PendingRequestExists,

    #[error("Friend request not found.")]
    RequestNotFound,

    #[error("Only the receiver can accept or reject a friend request.")]
    NotReceiver,

    #[error("Friends service: {0}")]
    Internal(String),
}