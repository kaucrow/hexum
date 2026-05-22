use std::sync::Arc;

use anyhow::Result;
use axum::{Router, routing::{get, post, delete}};
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};
use owo_colors::OwoColorize;

use hexum::{
    AppState,
    get_config,
    prelude::*,
    config::EmailSender,
    telemetry,
    application::{
        ports::output::EmailPort,
        services::{AuthService, UserService}
    },
    infrastructure::{
        PostgresAdapter,
        RedisSessionAdapter,
        PasetoSecurityAdapter,
        RedisVerificationAdapter,
        LettreEmailAdapter,
        ResendEmailAdapter,
        OAuthAdapter,
    },
    presentation::http::{self, routes},
};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let config = Arc::new(get_config()?);

    // Init the tracing subscriber
    let (subscriber, _guard) = telemetry::get_subscriber(&config).await?;
    telemetry::init(subscriber);

    let pg_adapter = Arc::new(PostgresAdapter::new(&config).await?);
    let redis_session_adapter = Arc::new(RedisSessionAdapter::new(&config).await?);
    let paseto_security_adapter = Arc::new(PasetoSecurityAdapter::new()?);
    let oauth_adapter = Arc::new(OAuthAdapter::new(&config));

    let auth_service = AuthService::new(
        pg_adapter.clone(),
        redis_session_adapter.clone(),
        paseto_security_adapter.clone(),
        oauth_adapter,
    );

    let redis_verification_adapter = Arc::new(RedisVerificationAdapter::new(&config).await?);
    let email_adapter: Arc<dyn EmailPort> = if let EmailSender::Smtp(_) = config.email.sender {
        Arc::new(LettreEmailAdapter::new(&config)?)
    } else {
        Arc::new(ResendEmailAdapter::new(&config)?)
    };

    let user_service = UserService::new(
        pg_adapter,
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
        .route("/user/register", post(routes::user::register))
        .route("/user/verify", get(routes::user::verify))
        .route("/auth/local/login", post(routes::auth::local::login))
        .route("/auth/oauth/google/login", post(routes::auth::oauth::google_login))
        .route("/auth/oauth/github/login", post(routes::auth::oauth::github_login))
        .route("/auth/refresh-session", post(routes::auth::refresh_session));

    if config.api.enable_dev_endpoints {
        app = app
            .route("/user/verify-ui", get(routes::user::verify_ui))
            .route("/auth/oauth/login-ui", get(routes::auth::oauth::oauth_login_ui))
            .route("/auth/oauth/callback-ui", get(routes::auth::oauth::oauth_callback_ui))
            .route("/dashboard", get(routes::management::manager_dashboard))
            .route("/nuke", delete(routes::management::delete_database))
    }

    let app = app
        .merge(Scalar::with_url(format!("/{}", config.api.docs_endpoint), http::ApiDocs::openapi()))
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