use crate::{
    Config,
    prelude::*,
    features::auth,
    api::*,
};
use super::dtos::*;

#[utoipa::path(
    post,
    path = "/auth/logout",
    description = "Logs out a user.",
    responses(
        (status = 200, description = "Logout successful. Clears authentication cookies.", body=LogoutResponse, headers(
            ("Set-Cookie" = String, description = "Clears access_token and refresh_token cookies")
        )),
        (status = 500, description = "Internal Server Error")
    ),
    tags = ["Authentication"]
)]
pub async fn logout(
    State(config): State<Arc<Config>>,
    State(auth_service): State<Arc<dyn auth::UseCase>>,
    jar: CookieJar,
) -> Result<(CookieJar, Json<LogoutResponse>), ApiError> {
    info!("Logout requested");

    if let Some(cookie) = jar.get("refresh_token") {
        let _ = auth_service.logout_user(cookie.value()).await;
    }

    let access_cookie = build_removal_cookie("access_token", "/", &config.api.protocol);
    let refresh_cookie = build_removal_cookie("refresh_token", "/auth/refresh-session", &config.api.protocol);

    let response = LogoutResponse { message: "Logout successful.".to_string() };
    Ok((jar.add(access_cookie).add(refresh_cookie), Json(response)))
}