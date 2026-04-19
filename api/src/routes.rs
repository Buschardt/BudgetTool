use axum::routing::get;
use axum::{Json, Router};

use crate::core::AppState;
use crate::core::response::ApiResponse;
use crate::{auth, files, manual_entries, reports, rules};

async fn health() -> Json<ApiResponse<&'static str>> {
    ApiResponse::success("ok")
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .merge(meta_router())
        .merge(auth::router())
        .merge(files::router())
        .merge(rules::router())
        .merge(manual_entries::router())
        .merge(reports::router())
        .with_state(state)
}

fn meta_router() -> Router<AppState> {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/me", get(auth::me))
}
