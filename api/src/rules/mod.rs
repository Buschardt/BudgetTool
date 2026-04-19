pub mod generator;
pub mod handlers;
pub mod models;
pub mod service;

use crate::core::AppState;
use axum::Router;
use axum::routing::{get, post};

pub use handlers::{create, delete, get_one, list, preview, update};
pub use service::resolve_rules_path;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/rules-configs", get(list).post(create))
        .route(
            "/api/rules-configs/{id}",
            get(get_one).put(update).delete(delete),
        )
        .route("/api/rules-configs/{id}/preview", post(preview))
}
