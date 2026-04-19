pub mod handlers;
pub mod journals;

use crate::core::AppState;
use axum::Router;
use axum::routing::get;

pub use handlers::{balance, cashflow, income_statement, register};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/reports/balance", get(balance))
        .route("/api/reports/incomestatement", get(income_statement))
        .route("/api/reports/register", get(register))
        .route("/api/reports/cashflow", get(cashflow))
}
