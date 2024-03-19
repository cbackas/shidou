use askama::filters::urlencode;
use askama_axum::IntoResponse;
use axum::{
    extract::Query,
    http::{header::CACHE_CONTROL, HeaderMap, HeaderValue, StatusCode},
    response::Redirect,
    Json,
};
use axum_extra::extract::PrivateCookieJar;
use cookie::Cookie;
use serde_json::json;
use tracing::error;

use crate::{
    middleware::auth::build_expired_cookie,
    models,
    utils::{discord, env, jwt, strings},
};

fn redirect_with_cache_control(url: &str) -> impl IntoResponse {
    let mut response = Redirect::temporary(url).into_response();
    response.headers_mut().insert(
        CACHE_CONTROL,
        HeaderValue::from_static("max-age=10, public"),
    );
    response
}

pub async fn get_login_redirect(headers: HeaderMap) -> impl IntoResponse {
    let discord_config = env::get_discord_config();
    let host = strings::get_host_header(&headers, true);
    let redirect_uri = format!("{}/auth/callback", host);
    let redirect_uri = urlencode(redirect_uri).expect("failed to urlencode redirect_uri");
    let scopes = ["identify", "guilds"];

    let url = format!(
        "https://discord.com/oauth2/authorize?client_id={}&response_type=code&redirect_uri={}&scope={}",
        discord_config.client_id,
        redirect_uri,
        scopes.join("%20"),
    );

    redirect_with_cache_control(&url).into_response()
}

pub async fn logout() -> impl axum::response::IntoResponse {
    let mut response = Redirect::temporary("/").into_response();
    response.headers_mut().insert(
        CACHE_CONTROL,
        HeaderValue::from_static("max-age=10, public"),
    );

    let key = cookie::Key::from(env::get_cookie_encryption_key().as_bytes());
    let jar = PrivateCookieJar::new(key)
        .add(build_expired_cookie("auth_token"))
        .add(build_expired_cookie("user_id"));

    for cookie in jar.iter() {
        let cookie_value = cookie.encoded().to_string();
        response
            .headers_mut()
            .append("Set-Cookie", cookie_value.parse().unwrap());
    }

    (jar, Redirect::temporary("/")).into_response()
}

#[derive(serde::Deserialize)]
pub struct CallbackQuery {
    pub code: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_token: String,
    pub scope: String,
}

pub async fn callback(headers: HeaderMap, Query(query): Query<CallbackQuery>) -> impl IntoResponse {
    if query.error.is_some() {
        error!("Discord OAuth error: {:?}", query.error_description);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": query.error_description.unwrap() })),
        )
            .into_response();
    }
    let code = query.code.expect("code not found");

    let discord_config = env::get_discord_config();
    let host = strings::get_host_header(&headers, true);
    let redirect_uri = format!("{}/auth/callback", host);

    let client = reqwest::Client::new();

    let res = client
        .post("https://discord.com/api/oauth2/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&[
            ("client_id", discord_config.client_id),
            ("client_secret", discord_config.client_secret),
            ("redirect_uri", redirect_uri),
            ("code", code),
            ("scope", "identify+guilds".to_string()),
            ("grant_type", "authorization_code".to_string()),
        ])
        .send()
        .await;

    if let Err(e) = res {
        error!("Failed to get token: {:?}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response();
    }
    let res = res.unwrap();

    if !res.status().is_success() {
        let err_text = res.text().await.unwrap();
        error!("Failed to get token: {:?}", err_text);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": err_text })),
        )
            .into_response();
    }

    let token_response = res
        .json::<TokenResponse>()
        .await
        .expect("failed to parse token response");

    let user_info = discord::get_user_info_by_token(&token_response.access_token).await;
    if let Err(e) = user_info {
        error!("Failed to get user info: {:?}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response();
    }
    let user_info = user_info.unwrap();

    if !discord_config.guilds.is_empty() {
        if !user_info.has_any_guild(discord_config.guilds.as_ref()) {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({ "error": "You are not a member of an allowed Discord guild" })),
            )
                .into_response();
        }
    }

    let upserted_user = models::user::upsert_user(&user_info.id, &user_info.username).await;
    if let Err(e) = upserted_user {
        error!("Failed to upsert user: {:?}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response();
    }
    let upserted_user = upserted_user.unwrap();

    let mut response = Redirect::temporary("/").into_response();
    response.headers_mut().insert(
        CACHE_CONTROL,
        HeaderValue::from_static("max-age=10, public"),
    );

    let key = cookie::Key::from(env::get_cookie_encryption_key().as_bytes());
    let jwt = jwt::create_jwt(upserted_user.id);
    if jwt.is_err() {
        let err_text = jwt.err().unwrap();
        error!("Failed to create JWT: {:?}", err_text);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": err_text.to_string() })),
        )
            .into_response();
    }
    let jwt = jwt.unwrap();

    let jar = PrivateCookieJar::new(key)
        .add(Cookie::build(("auth_token", jwt)).path("/"))
        .add(Cookie::build(("user_id", upserted_user.id.to_string())).path("/"));

    for cookie in jar.iter() {
        let cookie_value = cookie.encoded().to_string();
        response
            .headers_mut()
            .append("Set-Cookie", cookie_value.parse().unwrap());
    }

    (jar, Redirect::temporary("/")).into_response()
}
