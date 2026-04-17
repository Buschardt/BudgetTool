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

/// Database row for the rules_configs table.
#[derive(Debug, sqlx::FromRow)]
pub struct RulesConfigRecord {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub description: String,
    pub config: String,
    pub disk_path: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// API-facing summary of a rules config (used in list responses — no config blob).
#[derive(Debug, Serialize)]
pub struct RulesConfigInfo {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<RulesConfigRecord> for RulesConfigInfo {
    fn from(r: RulesConfigRecord) -> Self {
        RulesConfigInfo {
            id: r.id,
            name: r.name,
            description: r.description,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

/// API-facing detail of a rules config (includes parsed config JSON).
#[derive(Debug, Serialize)]
pub struct RulesConfigDetail {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub config: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

impl From<RulesConfigRecord> for RulesConfigDetail {
    fn from(r: RulesConfigRecord) -> Self {
        let config: serde_json::Value = serde_json::from_str(&r.config)
            .unwrap_or(serde_json::Value::Object(Default::default()));
        RulesConfigDetail {
            id: r.id,
            name: r.name,
            description: r.description,
            config,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
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
