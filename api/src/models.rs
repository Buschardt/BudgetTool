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
}
