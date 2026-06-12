use crate::{
    Config,
    prelude::*,
    features::{auth, ratelimit},
    api::*,
};
use super::dtos::*;

#[utoipa::path(
    post,
    path = "/auth/local/login",
    description = "Logs in a user with username/email & password.",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse, headers(
            ("Set-Cookie" = String, description = "HTTP-only cookies for access_token and refresh_token")
        )),
        (status = 401, description = "Unauthorized. Invalid username/email or password"),
        (status = 422, description = "Validation Error"),
        (status = 429, description = "Too Many Requests"),
        (status = 500, description = "Internal Server Error"),
    ),
    tags = ["Authentication"]
)]
pub async fn login(
    State(config): State<Arc<Config>>,
    State(auth_service): State<Arc<dyn auth::UseCase>>,
    State(ratelimit): State<Arc<dyn ratelimit::UseCase>>,
    jar: CookieJar,
    ClientIp(client_ip): ClientIp,
    ValidatedJson(payload): ValidatedJson<LoginRequest>,
) -> Result<(CookieJar, Json<LoginResponse>), ApiError> {
    let identity = &payload.identity;

    info!("Login attempt for user '{}' from IP {}", identity, &client_ip);

    // ─── IP-based rate limiting ───
    ratelimit
        .check_ip_limit(&client_ip, "login")
        .await?;

    // ─── Identity lockout check ───
    ratelimit
        .check_login_lockout(identity)
        .await?;

    // ─── Attempt login ───
    match auth_service.login_user(identity, &payload.password).await {
        Ok(tokens) => {
            // Success: clear any failed attempt counters
            let _ = ratelimit.clear_login_failures(identity).await;

            let access_cookie = build_cookie("access_token", tokens.access_token, "/", &config.api.protocol);
            let refresh_cookie = build_cookie("refresh_token", tokens.refresh_token, "/auth/refresh-session", &config.api.protocol);

            info!("Login successful for user '{}'", identity);

            let response = LoginResponse { message: "Login successful.".to_string() };
            Ok((jar.add(access_cookie).add(refresh_cookie), Json(response)))
        }
        Err(e) => {
            // Failure: record the attempt
            let status = ratelimit
                .record_login_failure(identity)
                .await
                .map_err(|_| ApiError::Internal)?;

            warn!(
                "Login failed for '{}' (attempt {}). Locked: {}",
                identity, status.attempts, status.is_locked
            );

            Err(e.into())
        }
    }
}