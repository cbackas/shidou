use axum::{
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::models::redirect;

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

pub async fn post(Json(payload): Json<RedirectInput>) -> impl IntoResponse {
    let url = match &payload.url {
        url if url.starts_with("http://") || url.starts_with("https://") => url.to_owned(),
        _ => format!("http://{}", payload.url),
    };
    match redirect::save_new_redirect(&payload.key, &url).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json(json!({ "message": "Redirect created successfully" })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": err.to_string() })),
        )
            .into_response(),
    }
}

pub async fn put(Json(payload): Json<RedirectInput>) -> impl IntoResponse {
    let url = match &payload.url {
        url if url.starts_with("http://") || url.starts_with("https://") => url.to_owned(),
        _ => format!("http://{}", payload.url),
    };
    match redirect::update_redirect(&payload.key, &url).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "message": "Redirect updated successfully" })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": err.to_string() })),
        )
            .into_response(),
    }
}

pub async fn delete(Json(payload): Json<DeleteRedirectInput>) -> impl IntoResponse {
    match redirect::delete_redirect(&payload.key).await {
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
