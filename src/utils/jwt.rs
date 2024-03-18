use anyhow::Context;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};

use super::env;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct JWT {
    pub user_id: i64,
    pub exp: usize,
}

pub fn create_jwt(user_id: i64) -> anyhow::Result<String> {
    let enc_key = EncodingKey::from_ed_pem(&env::get_jwt_private())
        .context("Failed to create JWT private key")?;

    let exp = (Utc::now() + Duration::try_days(30).unwrap()).timestamp() as usize;
    let jwt = JWT { user_id, exp };
    let token = encode(&Header::new(Algorithm::EdDSA), &jwt, &enc_key)
        .context("Failed to create new JWT")?;

    Ok(token)
}
