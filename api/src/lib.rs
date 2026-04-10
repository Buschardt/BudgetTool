pub mod auth;
pub mod db;
pub mod error;
pub mod hledger;
pub mod models;
pub mod response;

use axum::routing::{get, post};
use axum::{Json, Router};
use models::{AppState, Claims};
use response::ApiResponse;

async fn health() -> Json<ApiResponse<&'static str>> {
    ApiResponse::success("ok")
}

async fn me(claims: Claims) -> Json<ApiResponse<Claims>> {
    ApiResponse::success(claims)
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/login", post(auth::login))
        .route("/api/me", get(me))
        .with_state(state)
}
