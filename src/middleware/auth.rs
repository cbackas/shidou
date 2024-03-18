use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};

use axum_extra::extract::PrivateCookieJar;
use cookie::{time, Cookie, Key};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use tracing::trace;

use crate::utils::{env, jwt::JWT};

#[derive(Debug, Clone)]
pub struct UserId(String);

pub async fn auth_cookie_middleware(
    headers: HeaderMap,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if let Some(user_id) = check_auth(&headers).await {
        let mut res = next.run(req).await;
        res.extensions_mut().insert(user_id);
        Ok(res)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn check_auth(headers: &HeaderMap) -> Option<UserId> {
    let key = Key::from(env::get_cookie_encryption_key().as_bytes());
    let jar = PrivateCookieJar::from_headers(headers, key);

    match jar.get("auth_token") {
        Some(auth_token) => {
            let jwt_secret = DecodingKey::from_ed_pem(&env::get_jwt_public())
                .expect("Failed to create JWT public key");
            let decoded_token = decode::<JWT>(
                auth_token.value(),
                &jwt_secret,
                &Validation::new(Algorithm::EdDSA),
            );
            if decoded_token.is_err() {
                trace!("Failed to decode JWT: {:?}", decoded_token);
                return None;
            }

            if let Some(user_id) = jar.get("user_id") {
                return Some(UserId(user_id.value().to_string()));
            } else {
                return None;
            }
        }
        None => {
            return None;
        }
    }
}

pub fn build_expired_cookie(name: &str) -> Cookie {
    Cookie::build((name, "deleted"))
        .path("/")
        .expires(time::OffsetDateTime::now_utc() - time::Duration::days(365))
        .into()
}
