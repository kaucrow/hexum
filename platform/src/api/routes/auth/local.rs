use crate::{
    Config,
    prelude::*,
    features::auth,
    api::*,
};
use super::{
    build_cookie,
    dtos::*,
};

#[utoipa::path(
    post,
    path = "/auth/local/login",
    description = "Logs in a user with username/email & password.",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse, headers(
            ("Set-Cookie" = String, description = "HTTP-only cookies for access_token and refresh_token")
        )),
        (status = 401, description = "Unauthorized - Invalid username/email or password"),
        (status = 500, description = "Internal Server Error")
    ),
    tags = ["Authentication"]
)]
pub async fn login(
    State(config): State<Arc<Config>>,
    State(auth_service): State<Arc<dyn auth::UseCase>>,
    jar: CookieJar,
    Json(payload): Json<LoginRequest>,
) -> Result<(CookieJar, Json<LoginResponse>), ApiError> {
    info!("Login attempt for user `{}`", &payload.identity);

    let tokens = auth_service
        .login_user(&payload.identity, &payload.password)
        .await?;

    // Attach cookies. Access token goes to the root path ("/")
    let access_cookie = build_cookie("access_token", tokens.access_token, "/", &config.api.protocol);
    let refresh_cookie = build_cookie("refresh_token", tokens.refresh_token, "/auth/refresh-session", &config.api.protocol);

    info!("Login successful for user `{}`", &payload.identity);

    let response = LoginResponse { message: "Login successful".to_string() };
    Ok((jar.add(access_cookie).add(refresh_cookie), Json(response)))
}