use anyhow::{Context, Error};
use chrono::{DateTime, Utc};
use libsql::named_params;

use crate::database;

use super::date::custom_date_format;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[allow(dead_code)]
pub struct UserRow {
    pub id: i64,
    pub discord_snowflake: String,
    pub discord_username: String,
    #[serde(with = "custom_date_format")]
    pub created_utc: DateTime<Utc>,
    #[serde(with = "custom_date_format")]
    pub updated_utc: DateTime<Utc>,
}

pub async fn upsert_user(
    discord_snowflake: &str,
    discord_username: &str,
) -> anyhow::Result<UserRow> {
    let conn = database::get_conn().await;

    let result = conn
        .execute(
            "INSERT INTO users (discord_snowflake, discord_username) VALUES (:discord_snowflake, :discord_username)
            ON CONFLICT(discord_snowflake) DO UPDATE SET
            discord_username = excluded.discord_username,
            updated_utc = (strftime('%Y-%m-%d %H:%M:%S', 'now'))
            ",
            named_params! {
                ":discord_snowflake": discord_snowflake,
                ":discord_username": discord_username,
            },
        )
        .await
        .context("Failed to upsert user into database")?;

    if result == 1 {
        return get_user_by_discord_id(discord_snowflake).await;
    } else {
        return Err(Error::msg("Failed to upsert user into database"));
    }
}

pub async fn get_user_by_discord_id(snowflake: &str) -> anyhow::Result<UserRow> {
    let conn = database::get_conn().await;

    let mut result = conn
        .query(
            &"
            SELECT * FROM users
            WHERE discord_snowflake = :discord_snowflake
			LIMIT 1
        ",
            named_params!(
                ":discord_snowflake": snowflake,
            ),
        )
        .await
        .context("Failed to insert new redirect into database")?;

    match result.next().await {
        Ok(Some(row)) => {
            let row = libsql::de::from_row::<_>(&row)?;
            Ok(row)
        }
        Err(e) => return Err(anyhow::anyhow!("Failed to get user by discord id: {}", e)),
        Ok(None) => return Err(anyhow::anyhow!("Failed to get user by discord id")),
    }
}
