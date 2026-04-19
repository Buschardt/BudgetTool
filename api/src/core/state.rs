use std::path::PathBuf;

use sqlx::sqlite::SqlitePool;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub jwt_secret: String,
    pub data_dir: PathBuf,
}
