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
        CREATE TABLE IF NOT EXISTS repositories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT NOT NULL UNIQUE,
            name TEXT,
            docker_image_name TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        CREATE TABLE IF NOT EXISTS notifications (
            id TEXT PRIMARY KEY,
            type TEXT NOT NULL,
            title TEXT NOT NULL,
            detail TEXT NOT NULL,
            status TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            duration INTEGER,
            server_name TEXT
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // Migration for existing DB
    let _ = sqlx::query("ALTER TABLE repositories ADD COLUMN docker_image_name TEXT")
        .execute(&pool)
        .await;

    let _ = sqlx::query("ALTER TABLE repositories ADD COLUMN default_server_id INTEGER")
        .execute(&pool)
        .await;

    let _ = sqlx::query("ALTER TABLE repositories ADD COLUMN default_compose_path TEXT")
        .execute(&pool)
        .await;

    Ok(pool)
}
