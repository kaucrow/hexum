use crate::prelude::*;
use crate::features::friends;
use super::*;

#[derive(Clone)]
pub struct Service {
    messages: Arc<dyn Port>,
    friends: Arc<dyn friends::Port>,
}

impl Service {
    pub fn new(messages: Arc<dyn Port>, friends: Arc<dyn friends::Port>) -> Self {
        Self { messages, friends }
    }

    /// Checks that two users are friends by looking for an accepted friend_request.
    async fn are_friends(&self, user_a: Uuid, user_b: Uuid) -> Result<bool, UseCaseError> {
        let friends_a = self.friends.get_friends(user_a).await?;
        Ok(friends_a.iter().any(|f| f.id == user_b))
    }
}

#[async_trait]
impl UseCase for Service {
    async fn send_message(
        &self,
        sender_id: Uuid,
        receiver_id: Uuid,
        message_type: MessageType,
        content: &str,
    ) -> Result<Message, UseCaseError> {
        if sender_id == receiver_id {
            return Err(UseCaseError::CannotMessageYourself);
        }

        if !self.are_friends(sender_id, receiver_id).await? {
            return Err(UseCaseError::NotFriends);
        }

        let id = Uuid::new_v4();
        let message = self.messages.insert_message(id, sender_id, receiver_id, message_type, content).await?;

        Ok(message)
    }

    async fn get_conversation(
        &self,
        user_id: Uuid,
        friend_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Message>, UseCaseError> {
        if !self.are_friends(user_id, friend_id).await? {
            return Err(UseCaseError::NotFriends);
        }

        Ok(self.messages.get_conversation(user_id, friend_id, limit, offset).await?)
    }
}

impl From<PortError> for UseCaseError {
    fn from(e: PortError) -> Self {
        match e {
            PortError::Internal(s) => UseCaseError::Internal(s),
        }
    }
}

impl From<friends::PortError> for UseCaseError {
    fn from(e: friends::PortError) -> Self {
        UseCaseError::Internal(e.to_string())
    }
}