use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use tracing::info;

pub async fn init_pool(database_url: &str) -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .expect("failed to connect to SQLite");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to run migrations");

    info!("database ready");
    pool
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn migration_creates_users_table() {
        let pool = init_pool("sqlite::memory:").await;
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='users'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(row.0, 1);
    }
}
