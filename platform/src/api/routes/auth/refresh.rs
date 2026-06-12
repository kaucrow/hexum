use crate::{
    Config,
    prelude::*,
    features::{auth, ratelimit},
    api::*,
};

#[utoipa::path(
    post,
    path = "/auth/refresh-session",
    description = "Generates a new access token using the refresh token.",
    responses(
        (status = 200, description = "Token refreshed successfully", headers(
            ("Set-Cookie" = String, description = "Updated HTTP-only cookies for access_token and refresh_token")
        )),
        (status = 401, description = "Unauthorized - Missing, invalid, or expired refresh token cookie"),
        (status = 429, description = "Too Many Requests"),
        (status = 500, description = "Internal Server Error")
    ),
    tags = ["Authentication"]
)]
pub async fn refresh_session(
    State(config): State<Arc<Config>>,
    State(auth_service): State<Arc<dyn auth::UseCase>>,
    State(ratelimit): State<Arc<dyn ratelimit::UseCase>>,
    jar: CookieJar,
    ClientIp(client_ip): ClientIp,
) -> Result<CookieJar, ApiError> {
    // ── IP-based rate limiting ──
    ratelimit
        .check_ip_limit(&client_ip, "refresh")
        .await?;

    info!("Session refresh requested");

    // Get the refresh token from the cookie
    let refresh_token = jar.get("refresh_token")
        .map(|c| c.value().to_string())
        .ok_or(ApiError::Unauthorized("The refresh token is missing".to_string()))?;

    let new_tokens = auth_service
        .refresh_session(&refresh_token)
        .await?;

    // Return the updated cookies
    let access_cookie = build_cookie("access_token", new_tokens.access_token, "/", &config.api.protocol);
    let refresh_cookie = build_cookie("refresh_token", new_tokens.refresh_token, "/auth/refresh-session", &config.api.protocol);

    Ok(jar.add(access_cookie).add(refresh_cookie))
}