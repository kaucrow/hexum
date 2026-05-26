use std::sync::Arc;

use axum::Router;
use platform::features::security::Port;
use sqlx::PgPool;
use tokio::sync::OnceCell;
use uuid::Uuid;

/// A test application instance with its own server bound to a random port.
pub struct TestApp {
    /// Base URL of the spawned server
    pub address: String,
    /// Reqwest client with cookie store enabled (handles Set-Cookie automatically)
    pub client: reqwest::Client,
    /// PostgreSQL pool connected to the shared test database
    pub pool: PgPool,
    /// Loaded application config (development)
    pub config: Arc<platform::Config>,
    /// Redis connection manager for direct token lookups
    pub redis_conn: ::redis::aio::ConnectionManager,
}

impl TestApp {
    /// Convenience: builds a full URL relative to this server.
    pub fn url(&self, path: &str) -> String {
        let path = path.trim_start_matches('/');
        format!("{}/{}", self.address, path)
    }

    /// Sends a POST request with a JSON body.
    pub async fn post_json(&self, path: &str, body: &impl serde::Serialize) -> reqwest::Response {
        self.client
            .post(self.url(path))
            .json(body)
            .send()
            .await
            .expect("POST request failed")
    }

    /// Sends a GET request.
    pub async fn get(&self, path: &str) -> reqwest::Response {
        self.client
            .get(self.url(path))
            .send()
            .await
            .expect("GET request failed")
    }

    // =======================================================
    //  Database helpers
    // =======================================================

    /// Deletes a user (and their authenticators via CASCADE) by email.
    pub async fn delete_user_by_email(&self, email: &str) {
        sqlx::query("DELETE FROM platform.user WHERE email = $1")
            .bind(email)
            .execute(&self.pool)
            .await
            .expect("Failed to clean up user");
    }

    /// Returns the user_id for a given email, if the user exists.
    pub async fn get_user_id_by_email(&self, email: &str) -> Option<Uuid> {
        sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM platform.user WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .expect("Failed to query user by email")
    }

    /// Returns whether the local authenticator for a user is verified.
    pub async fn is_user_verified(&self, user_id: &Uuid) -> Option<bool> {
        sqlx::query_scalar::<_, Option<bool>>(
            "SELECT is_verified FROM platform.user_authenticator \
             WHERE user_id = $1 AND provider = 'Local'",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .expect("Failed to query authenticator")
        .flatten()
    }

    // =======================================================
    //  Redis helpers
    // =======================================================

    /// Deletes a verification token from Redis by token value.
    pub async fn delete_verification_token(&self, token: &str) {
        let key = format!("verify:{token}");
        let mut conn = self.redis_conn.clone();
        let _: () = ::redis::cmd("DEL")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .expect("Failed to delete Redis key");
    }

    /// Stores a verification token in Redis for the given user_id.
    /// Uses the same key format as the application (`verify:{token}`).
    pub async fn store_verification_token(&self, user_id: &Uuid, token: &str) {
        let key = format!("verify:{token}");
        let mut conn = self.redis_conn.clone();
        let _: () = ::redis::cmd("SET")
            .arg(&key)
            .arg(user_id.to_string())
            .arg("EX")
            .arg(3600u64)
            .query_async(&mut conn)
            .await
            .expect("Failed to store verification token in Redis");
    }

    /// Generates a unique test username (alphanumeric, lowercase).
    pub fn unique_username() -> String {
        let suffix = Uuid::new_v4().to_string().replace('-', "");
        format!("testuser{suffix}")
    }

    /// Generates a unique test email.
    pub fn unique_email() -> String {
        let suffix = Uuid::new_v4().to_string().replace('-', ".");
        format!("test.{suffix}@hexum-test.example")
    }

    /// A valid password that passes all validation rules (min 12 chars, number, symbol).
    pub fn valid_password() -> String {
        "TestPass123!@#".to_string()
    }

    /// Creates a user directly in the database with a known password hash.
    /// The authenticator is marked as verified so the user can log in.
    /// Returns the user's UUID.
    pub async fn seed_verified_user(&self, username: &str, email: &str) -> Uuid {
        let user_id = Uuid::new_v4();
        let hash = Self::hash_known_password();
        sqlx::query(
            "INSERT INTO platform.user (id, username, email, roles, is_active) \
             VALUES ($1, $2, $3, ARRAY['user'], true)"
        )
        .bind(user_id)
        .bind(username)
        .bind(email)
        .execute(&self.pool)
        .await
        .expect("Failed to seed user");
        sqlx::query(
            "INSERT INTO platform.user_authenticator \
             (id, user_id, provider, passwd, is_verified) \
             VALUES ($1, $2, 'Local', $3, true)"
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(&hash)
        .execute(&self.pool)
        .await
        .expect("Failed to seed authenticator");
        user_id
    }

    /// Creates a user directly in the database with a known password hash
    /// but with an un-verified authenticator. Useful for testing login-without-verify.
    pub async fn seed_unverified_user_with_password(&self, username: &str, email: &str) -> Uuid {
        let user_id = Uuid::new_v4();
        let hash = Self::hash_known_password();
        sqlx::query(
            "INSERT INTO platform.user (id, username, email, roles, is_active) \
             VALUES ($1, $2, $3, ARRAY['user'], true)"
        )
        .bind(user_id)
        .bind(username)
        .bind(email)
        .execute(&self.pool)
        .await
        .expect("Failed to seed user");
        sqlx::query(
            "INSERT INTO platform.user_authenticator \
             (id, user_id, provider, passwd, is_verified) \
             VALUES ($1, $2, 'Local', $3, false)"
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(&hash)
        .execute(&self.pool)
        .await
        .expect("Failed to seed authenticator");
        user_id
    }

    /// Creates a user directly in the database without authentication.
    /// Useful for testing conflict detection (duplicate username/email).
    pub async fn seed_unverified_user(&self, username: &str, email: &str) -> Uuid {
        let user_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO platform.user (id, username, email, roles, is_active) \
             VALUES ($1, $2, $3, ARRAY['user'], true)"
        )
        .bind(user_id)
        .bind(username)
        .bind(email)
        .execute(&self.pool)
        .await
        .expect("Failed to seed user");
        user_id
    }

    /// Hash the known test password using the real PasetoAdapter.
    fn hash_known_password() -> String {
        let security = platform::features::security::PasetoAdapter::new()
            .expect("Failed to create PasetoAdapter");
        let password = platform::features::user::Password::new("TestPass123!@#".to_string())
            .expect("Failed to create Password");
        security.hash_password(&password)
            .expect("Failed to hash password")
    }
}

// =======================================================
//  Migrations (runs once per process)
// =======================================================

static MIGRATIONS_DONE: OnceCell<()> = OnceCell::const_new();

async fn ensure_migrations_run(pool: &PgPool) {
    let pool = pool.clone();
    MIGRATIONS_DONE
        .get_or_init(|| async move {
            platform::postgres::run_migrations(&pool)
                .await
                .expect("Platform migrations failed");
            business::postgres::run_migrations(&pool)
                .await
                .expect("Business migrations failed");
        })
        .await;
}

// =======================================================
//  Server spawning
// =======================================================

/// Spawns a fresh server on a random port and returns a [`TestApp`] handle.
///
/// Each call creates an independent server with its own adapters.
/// Migrations run **once** globally via [`OnceCell`].
pub async fn spawn_test_app() -> TestApp {
    // This is test-only code, called before any other threads are spawned.

    // ── Tracing subscriber ────────────────────────────────────
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info")
        .try_init();

    unsafe {
        std::env::set_var("APP_ENV", "development");
        // Override Docker-internal hostnames so tests can reach Postgres/Redis
        // from the host machine via localhost.
        std::env::set_var("HEXUM_POSTGRESQL__HOST", "127.0.0.1");
        std::env::set_var("HEXUM_REDIS__HOST", "127.0.0.1");
    }

    let config = Arc::new(platform::get_config().expect("Failed to load config"));

    // ── Postgres pool ─────────────────────────────────────────
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(config.postgres.pool_max_conn)
        .acquire_timeout(std::time::Duration::from_secs(3))
        .connect(&config.postgres.url())
        .await
        .expect("Failed to connect to PostgreSQL");

    // Run migrations once
    ensure_migrations_run(&pool).await;

    // ── Redis connection (for direct token lookups ─────────────
    let redis_client =
        ::redis::Client::open(config.redis.url()).expect("Invalid Redis URL");
    let redis_conn = ::redis::aio::ConnectionManager::new(redis_client)
        .await
        .expect("Failed to connect to Redis");

    // ── Build adapters manually (bypass platform::init to avoid   ──
    // ── setting a global tracing subscriber more than once)       ──
    let pg_user_adapter = Arc::new(platform::features::user::PostgresAdapter::new(
        pool.clone(),
    ));
    let redis_session_adapter = Arc::new(
        platform::features::session::RedisAdapter::new(&config)
            .await
            .expect("Failed to create session RedisAdapter"),
    );
    let paseto_security_adapter = Arc::new(
        platform::features::security::PasetoAdapter::new()
            .expect("Failed to create PasetoAdapter"),
    );
    let oauth_adapter =
        Arc::new(platform::features::oauth::OAuthAdapter::new(&config));

    let auth_service = platform::features::auth::Service::new(
        pg_user_adapter.clone(),
        redis_session_adapter.clone(),
        paseto_security_adapter.clone(),
        oauth_adapter,
    );

    let redis_verification_adapter = Arc::new(
        platform::features::verification::RedisAdapter::new(&config)
            .await
            .expect("Failed to create verification RedisAdapter"),
    );

    // Determine which email adapter to use
    let email_adapter: Arc<dyn platform::features::email::Port> =
        if config.email.sender.smtp_config().is_ok() {
            Arc::new(
                platform::features::email::LettreAdapter::new(&config)
                    .expect("Failed to create LettreAdapter"),
            )
        } else {
            Arc::new(
                platform::features::email::ResendAdapter::new(&config)
                    .expect("Failed to create ResendAdapter"),
            )
        };

    let user_service = platform::features::user::Service::new(
        pg_user_adapter,
        redis_verification_adapter,
        paseto_security_adapter,
        email_adapter,
    );

    let platform_state = platform::PlatformState {
        config: config.clone(),
        auth: Arc::new(auth_service),
        user: Arc::new(user_service),
    };

    // ── Business layer (uses business::init) ───────────────────
    let business_state = business::init(pool.clone())
        .await
        .expect("Failed to init business state");

    // ── Router ─────────────────────────────────────────────────
    let platform_router =
        platform::api::router(platform_state, config.api.enable_dev_endpoints);
    let business_router = business::api::router(business_state);
    let app = Router::new()
        .merge(platform_router)
        .merge(business_router);

    // ── Bind to a random port ──────────────────────────────────
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to random port");
    let addr = listener.local_addr().expect("Failed to get bound address");

    // ── Spawn the server in the background ──────────────────────
    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("Server failed");
    });

    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .expect("Failed to build reqwest Client");

    TestApp {
        address: format!("http://{}:{}", addr.ip(), addr.port()),
        client,
        pool,
        config,
        redis_conn,
    }
}