use super::dtos::*;
use crate::{
    prelude::*,
    api::*,
    BusinessState,
};

// ─── Get conversation history ───

#[utoipa::path(
    get,
    path = "/messages/{friend_id}",
    description = "Returns paginated message history with the specified friend, newest first.",
    params(
        ("friend_id" = Uuid, Path, description = "The friend's user ID"),
        PaginationQuery,
    ),
    responses(
        (status = 200, description = "Conversation messages", body = ConversationResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Not friends with this user"),
        (status = 500, description = "Internal Server Error"),
    ),
    security(("cookie_auth" = [])),
    tags = ["Messages"]
)]
pub async fn get_conversation(
    auth: AuthenticatedUser,
    State(state): State<BusinessState>,
    Path(friend_id): Path<Uuid>,
    ValidatedQuery(pagination): ValidatedQuery<PaginationQuery>,
) -> Result<Json<ConversationResponse>, ApiError> {
    info!(
        "User '{}' fetching conversation with '{}'",
        &auth.user_id, &friend_id
    );

    let limit = pagination.limit();
    let offset = pagination.offset();

    let messages = state.messages
        .get_conversation(auth.user_id, friend_id, limit, offset)
        .await?;

    let response = ConversationResponse {
        messages: messages.into_iter().map(MessageResponse::from).collect(),
        limit,
        offset,
    };

    Ok(Json(response))
}