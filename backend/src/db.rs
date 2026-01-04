use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Pool, Sqlite};
use std::fs;
use std::path::Path;

const DB_URL: &str = "sqlite://onepanel.db?mode=rwc";

pub type DbPool = Pool<Sqlite>;

pub async fn init_db() -> Result<DbPool, sqlx::Error> {
    if !Path::new("onepanel.db").exists() {
        fs::File::create("onepanel.db").unwrap();
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(DB_URL)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS servers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            host TEXT NOT NULL,
            port INTEGER NOT NULL,
            api_key TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}
