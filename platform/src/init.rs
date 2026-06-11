use crate::{
    Config,
    PlatformState,
    prelude::*,
    config::EmailSender,
    telemetry,
    features::*,
};

pub async fn init(
    pool: sqlx::PgPool,
    redis_conn: redis::aio::ConnectionManager,
    config: Arc<Config>
) -> Result<PlatformState, anyhow::Error> {
    // Init the tracing subscriber
    let (subscriber, _guard) = telemetry::get_subscriber(&config).await?;
    telemetry::init(subscriber);

    let pg_user_adapter = Arc::new(user::PostgresAdapter::new(pool));
    let redis_session_adapter = Arc::new(session::RedisAdapter::new(redis_conn.clone()).await?);
    let paseto_security_adapter = Arc::new(security::PasetoAdapter::new()?);
    let oauth_adapter = Arc::new(oauth::OAuthAdapter::new(&config));

    let auth_service = auth::Service::new(
        pg_user_adapter.clone(),
        redis_session_adapter.clone(),
        paseto_security_adapter.clone(),
        oauth_adapter,
    );

    let redis_verification_adapter = Arc::new(verification::RedisAdapter::new(redis_conn).await?);

    let email_adapter: Arc<dyn email::Port> = match config.email.sender {
        EmailSender::Smtp(_) => Arc::new(email::LettreAdapter::new(&config)?),
        _ => Arc::new(email::ResendAdapter::new(&config)?),
    };

    let user_service = user::Service::new(
        pg_user_adapter,
        redis_verification_adapter,
        paseto_security_adapter,
        email_adapter,
    );

    Ok(PlatformState {
        config,
        auth: Arc::new(auth_service),
        user: Arc::new(user_service),
    })
}