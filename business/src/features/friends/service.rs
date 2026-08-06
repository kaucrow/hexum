use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use super::*;

#[derive(Clone)]
pub struct Service {
    friends: Arc<dyn Port>,
}

impl Service {
    pub fn new(friends: Arc<dyn Port>) -> Self {
        Self { friends }
    }
}

#[async_trait]
impl UseCase for Service {
    async fn list_users(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<UserSummary>, UseCaseError> {
        Ok(self.friends.list_strangers(user_id, limit, offset).await?)
    }

    async fn send_friend_request(
        &self,
        sender_id: Uuid,
        receiver_id: Uuid,
    ) -> Result<FriendRequest, UseCaseError> {
        if sender_id == receiver_id {
            return Err(UseCaseError::CannotFriendYourself);
        }

        let id = Uuid::new_v4();
        let request = self.friends.create_request(id, sender_id, receiver_id).await?;

        Ok(request)
    }

    async fn get_sent_requests(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<FriendRequest>, UseCaseError> {
        Ok(self.friends.get_sent_requests(user_id).await?)
    }

    async fn get_received_requests(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<FriendRequest>, UseCaseError> {
        Ok(self.friends.get_received_requests(user_id).await?)
    }

    async fn accept_friend_request(
        &self,
        request_id: Uuid,
        current_user_id: Uuid,
    ) -> Result<FriendRequest, UseCaseError> {
        let request = self.friends
            .accept_request(request_id, current_user_id)
            .await?
            .ok_or(UseCaseError::RequestNotFound)?;

        Ok(request)
    }

    async fn reject_friend_request(
        &self,
        request_id: Uuid,
        current_user_id: Uuid,
    ) -> Result<FriendRequest, UseCaseError> {
        let request = self.friends
            .reject_request(request_id, current_user_id)
            .await?
            .ok_or(UseCaseError::RequestNotFound)?;

        Ok(request)
    }

    async fn get_friends(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<UserSummary>, UseCaseError> {
        Ok(self.friends.get_friends(user_id).await?)
    }
}

impl From<PortError> for UseCaseError {
    fn from(e: PortError) -> Self {
        match e {
            PortError::Conflict(c) => match c {
                ConflictError::PendingRequestExists => UseCaseError::PendingRequestExists,
            },
            PortError::Internal(s) => UseCaseError::Internal(s),
        }
    }
}