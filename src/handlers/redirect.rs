use askama_axum::IntoResponse;
use axum::{
    extract::Path,
    http::{header::CACHE_CONTROL, HeaderValue, StatusCode},
    response::Redirect,
};

use crate::models::{self, redirect::inc_visits};

fn redirect_with_cache_control(url: &str) -> impl IntoResponse {
    let mut response = Redirect::temporary(url).into_response();
    response.headers_mut().insert(
        CACHE_CONTROL,
        HeaderValue::from_static("max-age=180, public"),
    );
    response
}

pub async fn get(Path(path): Path<String>) -> impl axum::response::IntoResponse {
    let redirects = models::redirect::get_all_redirects().await;
    let redirect = match &redirects {
        Ok(redirects) => redirects.iter().find(|r| r.key == path),
        Err(_) => None,
    };
    match redirect {
        Some(redirect) => {
            let key = redirect.key.clone();
            tokio::spawn(async move {
                let _ = inc_visits(&key).await;
            });
            redirect_with_cache_control(&redirect.url).into_response()
        }
        None => (StatusCode::NOT_FOUND, "Not Found").into_response(),
    }
}
