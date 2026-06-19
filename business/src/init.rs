use crate::{
    BusinessState,
    get_config,
    prelude::*,
    features::*,
};

pub async fn init(
    pool: sqlx::PgPool,
    redis_conn: redis::aio::ConnectionManager,
) -> Result<BusinessState, anyhow::Error> {
    let config = Arc::new(get_config()?);

    // ── Base ──────────────────────────────────────────────────────────
    let pg_base_adapter = Arc::new(base::PostgresAdapter::new(pool.clone()));
    let base_service = Arc::new(base::Service::new(pg_base_adapter));

    // ── Search ────────────────────────────────────────────────────────
    let http_client = reqwest::Client::new();

    // Videogame API adapter. Holds auth credentials + access token,
    // provides `request<T>()` with 401 -> re-auth -> retry logic.
    let auth_data = videogame_api::VideogameApiAuthData {
        url: config.videogame_api.auth.url.clone(),
        client_id: config.videogame_api.auth.client_id.clone(),
        client_secret: config.videogame_api.auth.client_secret.clone(),
    };
    let igdb_port = videogame_api::IgdbAdapter::new(http_client.clone(), auth_data);

    let internal_search_adapter = Arc::new(search::PostgresAdapter::new(pool));
    let external_search_adapter = Arc::new(search::IgdbAdapter::new(igdb_port));
    let search_cache_adapter = Arc::new(search::RedisAdapter::new(redis_conn));

    let search_service = Arc::new(search::Service::new(
        internal_search_adapter,
        external_search_adapter,
        search_cache_adapter,
    ));

    Ok(BusinessState {
        config,
        base: base_service,
        search: search_service,
    })
}
