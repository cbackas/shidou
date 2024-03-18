use axum::{
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
    Extension,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::trace;

use crate::{middleware::auth::UserId, models::redirect, utils::strings};

#[derive(Serialize, Deserialize)]
pub struct RedirectInput {
    key: String,
    url: String,
}

#[derive(Serialize, Deserialize)]
pub struct DeleteRedirectInput {
    key: String,
}

pub async fn get() -> impl IntoResponse {
    match redirect::get_all_redirects().await {
        Ok(redirects) => Json(redirects).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": err.to_string() })),
        )
            .into_response(),
    }
}

pub async fn post(
    headers: HeaderMap,
    Extension(user_id): Extension<UserId>,
    Json(payload): Json<RedirectInput>,
) -> impl IntoResponse {
    let url = match &payload.url {
        url if url.starts_with("http://") || url.starts_with("https://") => url.to_owned(),
        _ => format!("http://{}", payload.url),
    };
    let host = strings::get_host_header(&headers, false);

    match redirect::save_new_redirect(&payload.key, &url, &host, user_id.into_i64()).await {
        Ok(redirect) => (StatusCode::OK, Json(redirect)).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": err.to_string() })),
        )
            .into_response(),
    }
}

pub async fn put(headers: HeaderMap, Json(payload): Json<RedirectInput>) -> impl IntoResponse {
    let url = match &payload.url {
        url if url.starts_with("http://") || url.starts_with("https://") => url.to_owned(),
        _ => format!("http://{}", payload.url),
    };
    let host = strings::get_host_header(&headers, false);

    match redirect::update_redirect(&payload.key, &url, &host).await {
        Ok(redirect) => (StatusCode::OK, Json(redirect)).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": err.to_string() })),
        )
            .into_response(),
    }
}

pub async fn delete(
    headers: HeaderMap,
    Json(payload): Json<DeleteRedirectInput>,
) -> impl IntoResponse {
    let host = strings::get_host_header(&headers, false);
    match redirect::delete_redirect(&payload.key, &host).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "message": "Redirect deleted successfully" })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": err.to_string() })),
        )
            .into_response(),
    }
}
