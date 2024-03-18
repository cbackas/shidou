use crate::{database::get_conn, models::date::custom_date_format};
use anyhow::Context;
use chrono::{DateTime, Utc};
use libsql::named_params;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[allow(dead_code)]
pub struct RedirectRow {
    id: i64,
    pub key: String,
    pub url: String,
    pub redirect_host: String,
    pub visits: u64,
    pub created_by: i64,
    #[serde(with = "custom_date_format")]
    pub created_utc: DateTime<Utc>,
    #[serde(with = "custom_date_format")]
    pub updated_utc: DateTime<Utc>,
}

pub async fn save_new_redirect(
    key: &str,
    url: &str,
    host: &str,
    created_by: i64,
) -> anyhow::Result<RedirectRow> {
    let conn = get_conn().await;

    let result = conn
        .execute(
            "insert into redirects (key, url, redirect_host, created_by) values (:key, :url, :redirect_host, :created_by)",
            named_params!(
                ":key": key,
                ":url": url,
                ":redirect_host": host,
                ":created_by": created_by
            ),
        )
        .await
        ?;

    match result {
        1 => Ok(get_redirect(key, host).await?),
        val => Err(anyhow::anyhow!(
            "Expected 1 row to be inserted, but got {}",
            val
        )),
    }
}

pub async fn update_redirect(key: &str, url: &str, host: &str) -> anyhow::Result<RedirectRow> {
    let conn = get_conn().await;

    let result = conn
        .execute(
            "update redirects set url = :url where key = :key",
            named_params!(
                ":key": key,
                ":url": url,
            ),
        )
        .await
        .context("Failed to update redirect in database")?;

    match result {
        1 => Ok(get_redirect(key, host).await?),
        _ => Err(anyhow::anyhow!("Failed to update redirect in database")),
    }
}

pub async fn delete_redirect(key: &str, host: &str) -> anyhow::Result<()> {
    let conn = get_conn().await;

    let result = conn
        .execute(
            "DELETE FROM redirects WHERE key = :key AND host = :host",
            named_params!(
                ":key": key,
                ":host": host,
            ),
        )
        .await
        .context("Failed to delete redirect from database")?;

    match result {
        1 => Ok(()),
        _ => Err(anyhow::anyhow!("Failed to delete redirect from database")),
    }
}

pub async fn get_redirect(key: &str, host: &str) -> anyhow::Result<RedirectRow> {
    let conn = get_conn().await;

    let mut result = conn
        .query(
            "SELECT * FROM redirects WHERE key = :key AND redirect_host = :redirect_host LIMIT 1",
            named_params!(
                ":key": key,
                ":redirect_host": host,
            ),
        )
        .await
        .context("Failed to get redirect from database")?;

    match result.next().await {
        Ok(Some(row)) => {
            let row = libsql::de::from_row::<_>(&row)?;
            Ok(row)
        }
        Err(e) => return Err(anyhow::anyhow!("Failed to get redirect by key: {}", e)),
        Ok(None) => return Err(anyhow::anyhow!("Failed to get redirect by key")),
    }
}

pub async fn get_all_redirects() -> anyhow::Result<Vec<RedirectRow>> {
    let conn = get_conn().await;

    let mut result = conn
        .query(&"select * from redirects", named_params!())
        .await
        .context("Failed to insert new redirect into database")?;

    let mut results: Vec<RedirectRow> = vec![];
    while let Ok(Some(r)) = result.next().await {
        let row = libsql::de::from_row::<_>(&r);
        if let Ok(row) = row {
            results.push(row);
            continue;
        } else {
            tracing::error!("Failed to deserialize row: {:?}", row);
        }
    }

    Ok(results)
}

pub async fn inc_visits(key: &str) -> anyhow::Result<()> {
    let conn = get_conn().await;

    let result = conn
        .execute(
            "update redirects set visits = visits + 1 where key = :key",
            named_params!(
                ":key": key,
            ),
        )
        .await
        .context("Failed to increment visits in database")?;

    match result {
        1 => Ok(()),
        _ => Err(anyhow::anyhow!("Failed to increment visits in database")),
    }
}
