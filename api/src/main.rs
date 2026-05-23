use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};
use owo_colors::OwoColorize;
use anyhow::Result;

use platform::{
    AppState,
    get_config,
    prelude::*,
    config::EmailSender,
    telemetry,
    features::*,
};

use api::docs::MasterDocs;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let config = Arc::new(get_config()?);

    // Init the tracing subscriber
    let (subscriber, _guard) = telemetry::get_subscriber(&config).await?;
    telemetry::init(subscriber);

    let pg_user_adapter = Arc::new(user::PostgresAdapter::new(&config).await?);
    let redis_session_adapter = Arc::new(session::RedisAdapter::new(&config).await?);
    let paseto_security_adapter = Arc::new(security::PasetoAdapter::new()?);
    let oauth_adapter = Arc::new(oauth::OAuthAdapter::new(&config));

    let auth_service = auth::Service::new(
        pg_user_adapter.clone(),
        redis_session_adapter.clone(),
        paseto_security_adapter.clone(),
        oauth_adapter,
    );

    let redis_verification_adapter = Arc::new(verification::RedisAdapter::new(&config).await?);
    let email_adapter: Arc<dyn email::Port> = if let EmailSender::Smtp(_) = config.email.sender {
        Arc::new(email::LettreAdapter::new(&config)?)
    } else {
        Arc::new(email::ResendAdapter::new(&config)?)
    };

    let user_service = user::Service::new(
        pg_user_adapter,
        redis_verification_adapter,
        paseto_security_adapter,
        email_adapter,
    );

    let state = AppState {
        config: config.clone(),
        auth: Arc::new(auth_service),
        user: Arc::new(user_service),
    };

    let mut app = Router::new()
        .route("/user/register", post(platform::routes::user::register))
        .route("/user/verify", get(platform::routes::user::verify))
        .route("/auth/local/login", post(platform::routes::auth::local::login))
        .route("/auth/oauth/google/login", post(platform::routes::auth::oauth::google_login))
        .route("/auth/oauth/github/login", post(platform::routes::auth::oauth::github_login))
        .route("/auth/refresh-session", post(platform::routes::auth::refresh_session))
        .route("/business-health", get(business::routes::health::health));

    if config.api.enable_dev_endpoints {
        app = app
            .route("/user/verify-ui", get(platform::routes::user::verify_ui))
            .route("/auth/oauth/login-ui", get(platform::routes::auth::oauth::oauth_login_ui))
            .route("/auth/oauth/callback-ui", get(platform::routes::auth::oauth::oauth_callback_ui))
    }

    let mut openapi = MasterDocs::openapi();
    openapi.merge(platform::api::Docs::openapi());
    openapi.merge(business::api::Docs::openapi());

    let app = app
        .merge(Scalar::with_url(format!("/{}", config.api.docs_endpoint), openapi))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(
        format!("{}:{}", config.api.host, config.api.port)
    ).await.unwrap();

    info!("Hexum is running on {} mode.", config.environment.cyan().bold());
    info!("API listening on {}", config.api.url().yellow());
    info!("View API docs at {}{}", config.api.url().yellow(), config.api.docs_endpoint.yellow().bold());

    axum::serve(listener, app).await?;

    Ok(())
}