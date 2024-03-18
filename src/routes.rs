use axum::{routing::get, Router};
use tower_http::services::{ServeDir, ServeFile};

use crate::handlers;
use crate::handlers::api;
use crate::handlers::auth;
use crate::handlers::components;
use crate::middleware::auth::auth_cookie_middleware;

pub fn main_router() -> Router {
    tracing::debug!("initializing router(s) ...");

    Router::new()
        .route("/", get(handlers::home::get))
        .route("/*key", get(handlers::redirect::get))
        .route("/healthcheck", get(|| async { "Ok" }))
        .merge(services_router())
        .nest("/auth", auth_router())
        .nest("/api", api_router())
        .nest("/ui", component_router())
}

/**
 * router for the static assets and such
**/
pub fn services_router() -> Router {
    let assets_path = match std::env::current_dir() {
        Ok(path) => path,
        Err(_) => std::path::PathBuf::from("./"),
    };

    let assets_path = format!("{}/assets", assets_path.to_str().unwrap());
    let favicon_path = format!("{}/favicon.ico", assets_path);
    let manifest_path = format!("{}/site.webmanifest", assets_path);

    Router::new()
        .nest_service("/assets", ServeDir::new(assets_path))
        .nest_service("/favicon.ico", ServeFile::new(favicon_path))
        .nest_service("/site.webmanifest", ServeFile::new(manifest_path))
}

fn auth_router() -> Router {
    Router::new()
        .route("/login", get(auth::get_login_redirect))
        .route("/logout", get(auth::logout))
        .route("/callback", get(auth::callback))
}

/**
 * router for our api routes and the strava setup routes
 **/
fn api_router() -> Router {
    Router::new()
        .route(
            "/redirect",
            get(api::redirect::get)
                .post(api::redirect::post)
                .put(api::redirect::put)
                .delete(api::redirect::delete),
        )
        .layer(axum::middleware::from_fn(auth_cookie_middleware))
}

fn component_router() -> Router {
    Router::new()
        .route(
            "/redirect_url_input",
            get(components::redirect_url_input::get),
        )
        .layer(axum::middleware::from_fn(auth_cookie_middleware))
}
