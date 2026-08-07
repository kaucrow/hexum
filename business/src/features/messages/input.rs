use super::{Message, MessageType};
use crate::prelude::*;

#[async_trait]
pub trait UseCase: Send + Sync + 'static {
    /// Send a message to a friend. Validates friendship, persists to DB.
    async fn send_message(
        &self,
        sender_id: Uuid,
        receiver_id: Uuid,
        message_type: MessageType,
        content: &str,
    ) -> Result<Message, UseCaseError>;

    /// Get paginated conversation history with a friend (newest first).
    async fn get_conversation(
        &self,
        user_id: Uuid,
        friend_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Message>, UseCaseError>;
}

#[derive(Error, Debug)]
pub enum UseCaseError {
    #[error("You are not friends with this user.")]
    NotFriends,

    #[error("Cannot send a message to yourself.")]
    CannotMessageYourself,

    #[error("Messages service: {0}")]
    Internal(String),
}
