use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;

/// Database row for the users table.
#[derive(sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password: String,
}

/// JWT claims — used for encoding tokens and as an Axum extractor.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,
    pub username: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
}

/// Shared application state passed to all handlers via Axum's State extractor.
#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub jwt_secret: String,
    pub data_dir: PathBuf,
}

/// Database row for the files table.
#[derive(Debug, sqlx::FromRow)]
pub struct FileRecord {
    pub id: i64,
    pub user_id: i64,
    pub filename: String,
    pub file_type: String,
    pub size_bytes: i64,
    pub disk_path: String,
    pub created_at: String,
}

/// API-facing file metadata (hides disk_path and user_id).
#[derive(Debug, Serialize)]
pub struct FileInfo {
    pub id: i64,
    pub filename: String,
    pub file_type: String,
    pub size_bytes: i64,
    pub created_at: String,
}

impl From<FileRecord> for FileInfo {
    fn from(r: FileRecord) -> Self {
        FileInfo {
            id: r.id,
            filename: r.filename,
            file_type: r.file_type,
            size_bytes: r.size_bytes,
            created_at: r.created_at,
        }
    }
}
