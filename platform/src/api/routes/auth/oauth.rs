use crate::{
    Config,
    prelude::*,
    features::{auth, ratelimit},
    api::*,
};
use super::dtos::*;

#[utoipa::path(
    post,
    path = "/auth/oauth/google/login",
    description = "Logs in a user using the code from Google's OAuth.",
    request_body = OAuthLoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse, headers(
            ("Set-Cookie" = String, description = "HTTP-only cookies for access_token and refresh_token")
        )),
        (status = 429, description = "Too Many Requests"),
        (status = 500, description = "Internal Server Error")
    ),
    tags = ["Authentication"]
)]
pub async fn google_login(
    State(config): State<Arc<Config>>,
    State(auth_service): State<Arc<dyn auth::UseCase>>,
    State(ratelimit): State<Arc<dyn ratelimit::UseCase>>,
    jar: CookieJar,
    ClientIp(client_ip): ClientIp,
    Json(payload): Json<OAuthLoginRequest>,
) -> Result<(CookieJar, Json<LoginResponse>), ApiError> {
    // ── IP-based rate limiting ──
    ratelimit
        .check_ip_limit(&client_ip, "oauth_google")
        .await?;

    info!("Google OAuth login requested with code {}`", &payload.code);

    let tokens = auth_service
        .login_user_via_google(&payload.code)
        .await?;

    // Attach cookies
    let access_cookie = build_cookie("access_token", tokens.access_token, "/", &config.api.protocol);
    let refresh_cookie = build_cookie("refresh_token", tokens.refresh_token, "/auth/refresh-session", &config.api.protocol);

    info!("Google OAuth login successful for code '{}'", &payload.code);

    let response = LoginResponse { message: "Login successful".to_string() };
    Ok((jar.add(access_cookie).add(refresh_cookie), Json(response)))
}

#[utoipa::path(
    post,
    path = "/auth/oauth/github/login",
    description = "Logs in a user using the code from GitHub's OAuth.",
    request_body = OAuthLoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse, headers(
            ("Set-Cookie" = String, description = "HTTP-only cookies for access_token and refresh_token")
        )),
        (status = 429, description = "Too Many Requests"),
        (status = 500, description = "Internal Server Error")
    ),
    tags = ["Authentication"]
)]
pub async fn github_login(
    State(config): State<Arc<Config>>,
    State(auth_service): State<Arc<dyn auth::UseCase>>,
    State(ratelimit): State<Arc<dyn ratelimit::UseCase>>,
    jar: CookieJar,
    ClientIp(client_ip): ClientIp,
    Json(payload): Json<OAuthLoginRequest>,
) -> Result<(CookieJar, Json<LoginResponse>), ApiError> {
    // ── IP-based rate limiting ──
    ratelimit
        .check_ip_limit(&client_ip, "oauth_github")
        .await?;

    info!("GitHub OAuth login requested with code {}`", &payload.code);

    let tokens = auth_service
        .login_user_via_github(&payload.code)
        .await?;

    // Attach cookies
    let access_cookie = build_cookie("access_token", tokens.access_token, "/", &config.api.protocol);
    let refresh_cookie = build_cookie("refresh_token", tokens.refresh_token, "/auth/refresh-session", &config.api.protocol);

    info!("GitHub OAuth login successful for code '{}'", &payload.code);

    let response = LoginResponse { message: "Login successful".to_string() };
    Ok((jar.add(access_cookie).add(refresh_cookie), Json(response)))
}

#[derive(Template)]
#[template(path = "oauth_login.html")]
pub struct OAuthLoginTemplate<'a> {
    oauth_redirect_url: &'a str,
    google_client_id: &'a str,
    github_client_id: &'a str,
}

#[utoipa::path(
    get,
    path = "/auth/oauth/login-ui",
    description = "**[DEVELOPMENT ENDPOINT]** Returns a simple HTML page to test OAuth.",
    responses(
        (
            status = 200,
            description = "Returns the OAuth login page", 
            body = String,
            content_type = "text/html"
        ),
        (status = 500, description = "Internal Server Error: Template rendering failed")
    ),
    tags = ["Authentication"]
)]
pub async fn oauth_login_ui(
    State(config): State<Arc<Config>>,
) -> Result<impl IntoResponse, ApiError> {
    let template = OAuthLoginTemplate {
        oauth_redirect_url: &config.oauth.redirect_url(&config.frontend.url),
        google_client_id: &config.oauth.google.client_id,
        github_client_id: &config.oauth.github.client_id,
    };

    let html_content = template
        .render()
        .map_err(|_| ApiError::Internal)?;

    Ok(Html(html_content))
}

#[derive(Template)]
#[template(path = "oauth_callback.html")]
pub struct OAuthCallbackTemplate<'a> {
    pub login_ui_url: &'a str,
    pub google_login_uri: &'a str,
    pub github_login_uri: &'a str,
}

#[utoipa::path(
    get,
    path = "/auth/oauth/callback-ui",
    description = "**[DEVELOPMENT ENDPOINT]** Renders a landing page for OAuth's redirect, for testing purposes. Grabs the 'code' from the URL and exchanges it with the backend.",
    params(
        ("code" = String, Query, description = "The authorization code returned by OAuth")
    ),
    responses(
        (
            status = 200,
            description = "Returns the 'Processing Request' HTML page", 
            body = String,
            content_type = "text/html"
        ),
        (status = 500, description = "Internal Server Error: Template rendering failed")
    ),
    tags = ["Authentication"]
)]
pub async fn oauth_callback_ui(
    State(config): State<Arc<Config>>,
) -> Result<impl IntoResponse, ApiError> {
    let google_login_uri = &format!("{}{}",
        config.api.path_suffix,
        config.oauth.google.login_endpoint,
    );

    let github_login_uri = &format!("{}{}",
        config.api.path_suffix,
        config.oauth.github.login_endpoint,
    );

    let template = OAuthCallbackTemplate {
        login_ui_url: &config.oauth.login_ui_url(&config.frontend.url),
        google_login_uri,
        github_login_uri,
    };

    let html_content = template
        .render()
        .map_err(|_l| ApiError::Internal)?;

    Ok(Html(html_content))
}
