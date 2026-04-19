mod extractor;
pub mod handlers;
pub mod jwt;
pub mod models;

use crate::core::AppState;
use axum::Router;
use axum::routing::post;

pub use handlers::{login, me};
pub use models::Claims;

pub fn router() -> Router<AppState> {
    Router::new().route("/api/login", post(login))
}
