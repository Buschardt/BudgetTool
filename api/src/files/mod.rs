pub mod filename;
pub mod handlers;
pub mod models;

use crate::core::AppState;
use axum::Router;
use axum::routing::{get, post};

pub use handlers::{convert_csv, create_journal, delete, get_one, list, upload};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/files", get(list).post(upload))
        .route("/api/files/{id}", get(get_one).delete(delete))
        .route("/api/files/{id}/convert", post(convert_csv))
        .route("/api/journals", post(create_journal))
}
