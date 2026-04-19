use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;

use crate::auth::Claims;
use crate::core::AppState;
use crate::core::error::AppError;
use crate::core::hledger;
use crate::core::response::ApiResponse;
use crate::reports::journals::{build_args, filter_args, journal_args};

#[derive(Deserialize)]
pub struct ReportQuery {
    pub begin: Option<String>,
    pub end: Option<String>,
    pub period: Option<String>,
    pub depth: Option<u32>,
    pub account: Option<String>,
}

pub async fn balance(
    claims: Claims,
    State(state): State<AppState>,
    Query(query): Query<ReportQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let file_args = journal_args(&state.db, claims.sub).await?;
    let filter = filter_args(&query);
    let args = build_args("balance", &file_args, &filter);
    let data = hledger::run(&args).await?;
    Ok(ApiResponse::success(data))
}

pub async fn income_statement(
    claims: Claims,
    State(state): State<AppState>,
    Query(query): Query<ReportQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let file_args = journal_args(&state.db, claims.sub).await?;
    let filter = filter_args(&query);
    let args = build_args("incomestatement", &file_args, &filter);
    let data = hledger::run(&args).await?;
    Ok(ApiResponse::success(data))
}

pub async fn register(
    claims: Claims,
    State(state): State<AppState>,
    Query(query): Query<ReportQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let file_args = journal_args(&state.db, claims.sub).await?;
    let filter = filter_args(&query);
    let args = build_args("register", &file_args, &filter);
    let data = hledger::run(&args).await?;
    Ok(ApiResponse::success(data))
}

pub async fn cashflow(
    claims: Claims,
    State(state): State<AppState>,
    Query(query): Query<ReportQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let file_args = journal_args(&state.db, claims.sub).await?;
    let filter = filter_args(&query);
    let args = build_args("cashflow", &file_args, &filter);
    let data = hledger::run(&args).await?;
    Ok(ApiResponse::success(data))
}
