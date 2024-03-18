use std::env;

use tracing::error;

// use crate::utils::hash_string;

pub fn get_host_uri() -> String {
    match env::var("HOST") {
        Ok(host) => format!("https://{}", host),
        _ => match env::var("FLY_APP_NAME") {
            Ok(host) => format!("https://{}.fly.dev", host),
            _ => {
                format!("http://localhost:{}", get_port())
            }
        },
    }
}

pub fn get_port() -> u16 {
    let default_port: u16 = 8080;

    let port = match env::var("PORT") {
        Ok(port) => port,
        _ => default_port.to_string(),
    };
    let port: u16 = match port.parse::<_>() {
        Ok(port) => port,
        _ => {
            error!("Failed to parse PORT env var, using default");
            default_port
        }
    };

    port
}

#[derive(Debug, Clone)]
pub struct DiscordConfig {
    pub client_id: String,
    pub client_secret: String,
}

pub fn get_discord_config() -> DiscordConfig {
    DiscordConfig {
        client_id: match env::var("DISCORD_CLIENT_ID") {
            Ok(client_id) => client_id,
            _ => panic!("DISCORD_CLIENT_ID not set"),
        },
        client_secret: match env::var("DISCORD_CLIENT_SECRET") {
            Ok(client_secret) => client_secret,
            _ => panic!("DISCORD_CLIENT_SECRET not set"),
        },
    }
}

// pub fn get_db_encryption_key() -> String {
//     match env::var("DB_ENCRYPTION_KEY") {
//         Ok(key) => key,
//         _ => panic!("DB_ENCRYPTION_KEY not set"),
//     }
// }

/// Get the encryption key for cookies
/// The supplied key must be at least 512-bits (64 bytes). For security, the master key must be cryptographically random.
pub fn get_cookie_encryption_key() -> String {
    match env::var("COOKIE_ENCRYPTION_KEY") {
        Ok(key) => key,
        _ => panic!("COOKIE_ENCRYPTION_KEY not set"),
    }
}

pub fn get_jwt_public() -> Vec<u8> {
    match env::var("JWT_SECRET_PUBLIC") {
        Ok(key) => key.into_bytes(),
        _ => panic!("JWT_SECRET_PUBLIC not set"),
    }
}

pub fn get_jwt_private() -> Vec<u8> {
    match env::var("JWT_SECRET_PRIVATE") {
        Ok(key) => key.into_bytes(),
        _ => panic!("JWT_SECRET_PRIVATE not set"),
    }
}
