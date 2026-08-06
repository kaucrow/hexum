use utoipa::OpenApi;
use super::routes;

#[derive(OpenApi)]
#[openapi(
    paths(
        routes::health::health,
        routes::friends::list_users::list_users,
        routes::friends::list_users::list_friends,
        routes::friends::requests::send_friend_request,
        routes::friends::requests::get_sent_requests,
        routes::friends::requests::get_received_requests,
        routes::friends::requests::accept_friend_request,
        routes::friends::requests::reject_friend_request,
    ),
    components(
        schemas(
            // ==== Core ====
            routes::dtos::BusinessHealthResponse,

            // ==== Friends DTOs ====
            routes::friends::dtos::PaginationQuery,
            routes::friends::dtos::UserSummaryResponse,
            routes::friends::dtos::UserListResponse,
            routes::friends::dtos::FriendRequestResponse,
            routes::friends::dtos::FriendRequestListResponse,
            routes::friends::dtos::SendFriendRequestResponse,
            routes::friends::dtos::AcceptRejectResponse,
            routes::friends::dtos::FriendListResponse,
        )
    ),
)]
pub struct Docs;