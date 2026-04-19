pub mod filename;
pub mod handlers;
pub mod journal_settings;
pub mod models;

use crate::core::AppState;
use axum::Router;
use axum::routing::{get, post};

pub use handlers::{convert_csv, create_journal, delete, get_one, list, upload};
pub use journal_settings::{get_settings, update_settings};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/files", get(list).post(upload))
        .route("/api/files/{id}", get(get_one).delete(delete))
        .route("/api/files/{id}/convert", post(convert_csv))
        .route("/api/journals", post(create_journal))
        .route(
            "/api/journals/{id}/settings",
            get(get_settings).put(update_settings),
        )
}
