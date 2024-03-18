use askama_axum::IntoResponse;
use axum::http::HeaderMap;

use crate::middleware::auth::check_auth;

pub async fn get(headers: HeaderMap) -> impl axum::response::IntoResponse {
    match check_auth(&headers).await.is_some() {
        true => {
            let host = match headers
                .get("host")
                .expect("host header not found")
                .to_str()
                .unwrap()
            {
                "localhost:8080" => "http://localhost:8080/".to_string(),
                h => format!("https://{}/", h),
            };

            DashboardPage { host }.into_response()
        }
        false => LoginPage {}.into_response(),
    }
}

#[derive(askama::Template)]
#[template(path = "pages/dashboard.html")]
struct DashboardPage {
    host: String,
}

#[derive(askama::Template)]
#[template(path = "pages/login.html")]
struct LoginPage {}
