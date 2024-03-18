use anyhow::Context;
use libsql::Builder;
use std::env;
use tokio::sync::OnceCell;

static DATABASE: OnceCell<libsql::Database> = OnceCell::const_new();

async fn get_db() -> &'static libsql::Database {
    DATABASE
        .get_or_init(|| async {
            let db = Builder::new_remote_replica(
                env::var("LIBSQL_LOCAL_DB_PATH").unwrap_or("file:local_replica.db".to_string()),
                env::var("LIBSQL_CLIENT_URL").expect("Missing LIBSQL_CLIENT_URL"),
                env::var("LIBSQL_CLIENT_TOKEN").expect("Missing LIBSQL_CLIENT_TOKEN"),
            )
            .build()
            .await
            .expect("Failed to create database");

            db.sync()
                .await
                .expect("Failed to sync remote db to local disk");

            tracing::debug!("Initialized db");

            db
        })
        .await
}

pub async fn get_conn() -> libsql::Connection {
    get_db().await.connect().expect("Failed to connect to db")
}

// async fn sync() -> anyhow::Result<()> {
//     let db = get_db().await;
//     db.sync().await?;
//     tracing::trace!("Synced remote db to local disk");
//     Ok(())
// }

pub async fn init_tables() -> anyhow::Result<()> {
    let conn = get_conn().await;

    let tx = conn
        .transaction_with_behavior(libsql::TransactionBehavior::Immediate)
        .await?;

    // tx.execute("DROP TABLE IF EXISTS users", libsql::params!())
    //     .await?;
    // tx.execute("DROP TABLE IF EXISTS redirects", libsql::params!())
    //     .await?;

    //
    // Redirects table
    //
    tx.execute(
        "CREATE TABLE IF NOT EXISTS redirects (
                id INTEGER PRIMARY KEY,
                key TEXT UNIQUE,
                url TEXT,
		redirect_host TEXT,
	    	visits INTEGER DEFAULT 0,
		created_by INTEGER,
                created_utc REAL DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now')),
                updated_utc REAL DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now')),
		FOREIGN KEY(created_by) REFERENCES users(id)
            )",
        libsql::params!(),
    )
    .await
    .context("Failed to create redirects table")?;
    tx.execute(
        "CREATE INDEX IF NOT EXISTS idx_key ON redirects(key)",
        libsql::params!(),
    )
    .await
    .context("Failed to create index on redirects table")?;

    //
    // Users table
    //
    tx.execute(
        "CREATE TABLE IF NOT EXISTS users ( \
                id INTEGER PRIMARY KEY, \
		discord_snowflake TEXT UNIQUE, \
            	discord_username TEXT, \
                created_utc REAL DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now')), \
                updated_utc REAL DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now')) \
            )",
        libsql::params!(),
    )
    .await
    .context("Failed to create users table")?;
    tx.execute(
        "CREATE INDEX IF NOT EXISTS idx_discord_snowflake ON users(discord_snowflake)",
        libsql::params!(),
    )
    .await
    .context("Failed to create index on users table")?;

    tx.commit().await.context("Failed to commit transaction")?;

    Ok(())
}
