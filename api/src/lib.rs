pub mod auth;
pub mod db;
pub mod error;
pub mod files;
pub mod hledger;
pub mod manual_entries;
pub mod manual_entries_gen;
pub mod models;
pub mod reports;
pub mod response;
pub mod rules_configs;
pub mod rules_gen;

use axum::routing::{get, post, put};
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
        .route("/api/files", get(files::list).post(files::upload))
        .route("/api/files/{id}", get(files::get_one).delete(files::delete))
        .route("/api/files/{id}/convert", post(files::convert_csv))
        .route("/api/journals", post(files::create_journal))
        .route(
            "/api/rules-configs",
            get(rules_configs::list).post(rules_configs::create),
        )
        .route(
            "/api/rules-configs/{id}",
            get(rules_configs::get_one)
                .put(rules_configs::update)
                .delete(rules_configs::delete),
        )
        .route(
            "/api/rules-configs/{id}/preview",
            post(rules_configs::preview),
        )
        .route(
            "/api/prices",
            get(manual_entries::list_prices).post(manual_entries::create_price),
        )
        .route(
            "/api/prices/{id}",
            put(manual_entries::update_price).delete(manual_entries::delete_price),
        )
        .route(
            "/api/transactions",
            get(manual_entries::list_transactions).post(manual_entries::create_transaction),
        )
        .route(
            "/api/transactions/{id}",
            put(manual_entries::update_transaction).delete(manual_entries::delete_transaction),
        )
        .route(
            "/api/periodics",
            get(manual_entries::list_periodics).post(manual_entries::create_periodic),
        )
        .route(
            "/api/periodics/{id}",
            put(manual_entries::update_periodic).delete(manual_entries::delete_periodic),
        )
        .route("/api/reports/balance", get(reports::balance))
        .route(
            "/api/reports/incomestatement",
            get(reports::income_statement),
        )
        .route("/api/reports/register", get(reports::register))
        .route("/api/reports/cashflow", get(reports::cashflow))
        .with_state(state)
}
