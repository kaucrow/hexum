use utoipa::OpenApi;
use super::routes;

#[derive(OpenApi)]
#[openapi(
    paths(
        routes::health::health,
        routes::games::get::get_game,
        routes::games::search::search,
        routes::games::popular::popular_games,
        routes::platforms::get::get_platforms,
        routes::platforms::sync::sync_platforms,
        routes::critic::apply::apply_for_critic,
        routes::critic::list::list_applications,
        routes::critic::approve::approve_application,
        routes::critic::approve::reject_application,
        routes::review::submit::submit_review,
    ),
    components(
        schemas(
            // ─── Requests & Responses ───
            routes::dtos::BusinessHealthResponse,
            routes::games::dtos::GameSearchResultItemResponse,
            routes::games::dtos::GameSearchMeta,
            routes::games::dtos::GameSearchResponse,
            routes::games::dtos::GameResponse,
            routes::games::dtos::GamePlatformResponse,
            routes::games::dtos::PopularGameItemResponse,
            routes::games::dtos::PopularGamesMeta,
            routes::games::dtos::PopularGamesResponse,
            routes::platforms::dtos::PlatformResponse,
            routes::platforms::dtos::PlatformSyncResponse,
            // ─── Critic ───
            routes::critic::dtos::ApplyForCriticResponse,
            routes::critic::dtos::CriticApplicationResponse,
            routes::critic::dtos::ApproveApplicationResponse,
            routes::critic::dtos::RejectApplicationResponse,
            // ─── Reviews ───
            routes::review::dtos::SubmitReviewRequest,
            routes::review::dtos::SubmitReviewResponse,
        )
    ),
)]
pub struct Docs;