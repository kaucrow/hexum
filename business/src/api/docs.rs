use utoipa::OpenApi;
use super::routes;

#[derive(OpenApi)]
#[openapi(
    paths(
        routes::health::health,
        // Friends
        routes::friends::list_users,
        routes::friends::send_friend_request,
        routes::friends::get_sent_requests,
        routes::friends::get_received_requests,
        routes::friends::accept_friend_request,
        routes::friends::reject_friend_request,
        routes::friends::get_friends,
        // Messages
        routes::messages::get_conversation,
        routes::messages::ws_handler,
    ),
    components(
        schemas(
            // ==== Core ====
            routes::dtos::BusinessHealthResponse,

            // ==== Friends DTOs ====
            routes::friends::dtos::PaginationQuery,
            routes::friends::dtos::LastMessageResponse,
            routes::friends::dtos::UserSummaryResponse,
            routes::friends::dtos::UserListResponse,
            routes::friends::dtos::FriendRequestResponse,
            routes::friends::dtos::FriendRequestListResponse,
            routes::friends::dtos::SendFriendRequestResponse,
            routes::friends::dtos::AcceptRejectResponse,
            routes::friends::dtos::FriendListResponse,

            // ==== Messages DTOs ====
            routes::messages::dtos::MessageResponse,
            routes::messages::dtos::ConversationResponse,
        )
    ),
)]
pub struct Docs;
