use std::sync::Arc;

use axum::Router;
use utoipa::{OpenApi, openapi::Server};
use utoipa_scalar::{Scalar, Servable};
use tracing::info;
use owo_colors::OwoColorize;
use anyhow::Result;

use platform::{Environment, get_config};

use api::{
    docs::{self, MasterDocs},
    db::init_postgres_pool,
};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let config = Arc::new(get_config()?);

    let pool = init_postgres_pool(&config).await?;

    let platform_state = platform::init(pool.clone(), config.clone()).await?;
    let business_state = business::init(pool).await?;

    let platform_router = platform::api::router(platform_state, config.api.enable_dev_endpoints);
    let business_router = business::api::router(business_state);

    let mut openapi = MasterDocs::openapi();

    if config.environment == Environment::Production {
        openapi.servers = Some(vec![
            Server::new("/api")
        ]);
    }

    openapi.merge(platform::api::Docs::openapi());
    openapi.merge(business::api::Docs::openapi());

    let docs_auth_layer = docs::get_auth_layer(config.clone());

    let docs_router: Router<()> = Router::new()
        .merge(Scalar::with_url("/", openapi)) 
        .layer(docs_auth_layer);

    let app = Router::new()
        .merge(platform_router)
        .merge(business_router)
        .nest(&format!("/{}", config.api.docs_endpoint), docs_router);

    let listener = tokio::net::TcpListener::bind(
        format!("{}:{}", config.api.host, config.api.port)
    ).await.unwrap();

    info!("Hexum is running on {} mode.", config.environment.cyan().bold());
    info!("API listening on {}", config.api.url().yellow());
    info!("View API docs at {}{}", config.api.url().yellow(), config.api.docs_endpoint.yellow().bold());

    axum::serve(listener, app).await?;

    Ok(())
}