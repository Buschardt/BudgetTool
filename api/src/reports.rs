use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;
use sqlx::sqlite::SqlitePool;

use crate::error::AppError;
use crate::hledger;
use crate::models::{AppState, Claims};
use crate::response::ApiResponse;

#[derive(Deserialize)]
pub struct ReportQuery {
    pub begin: Option<String>,
    pub end: Option<String>,
    pub period: Option<String>,
    pub depth: Option<u32>,
    pub account: Option<String>,
}

/// Fetch all journal file paths for the user from the database.
/// Returns an error if the user has no journal files uploaded.
async fn journal_args(db: &SqlitePool, user_id: i64) -> Result<Vec<String>, AppError> {
    let rows: Vec<(String,)> =
        sqlx::query_as("SELECT disk_path FROM files WHERE user_id = ? AND file_type = 'journal'")
            .bind(user_id)
            .fetch_all(db)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

    if rows.is_empty() {
        return Err(AppError::BadRequest(
            "no journal files uploaded; upload a .journal file first".into(),
        ));
    }

    let mut args = Vec::with_capacity(rows.len() * 2);
    for (path,) in rows {
        args.push("-f".to_string());
        args.push(path);
    }
    Ok(args)
}

/// Convert ReportQuery fields into hledger CLI flags.
fn filter_args(query: &ReportQuery) -> Vec<String> {
    let mut args = Vec::new();
    if let Some(begin) = &query.begin {
        args.push("--begin".to_string());
        args.push(begin.clone());
    }
    if let Some(end) = &query.end {
        args.push("--end".to_string());
        args.push(end.clone());
    }
    if let Some(period) = &query.period {
        args.push("--period".to_string());
        args.push(period.clone());
    }
    if let Some(depth) = query.depth {
        args.push("--depth".to_string());
        args.push(depth.to_string());
    }
    if let Some(account) = &query.account {
        args.push(account.clone());
    }
    args
}

/// Build the full args slice for hledger from a subcommand, journal paths, and filters.
fn build_args<'a>(subcommand: &'a str, file_args: &'a [String], filter_args: &'a [String]) -> Vec<&'a str> {
    let mut args = vec![subcommand];
    for s in file_args {
        args.push(s.as_str());
    }
    for s in filter_args {
        args.push(s.as_str());
    }
    args
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

#[cfg(test)]
mod tests {
    use super::*;

    fn query(
        begin: Option<&str>,
        end: Option<&str>,
        period: Option<&str>,
        depth: Option<u32>,
        account: Option<&str>,
    ) -> ReportQuery {
        ReportQuery {
            begin: begin.map(String::from),
            end: end.map(String::from),
            period: period.map(String::from),
            depth,
            account: account.map(String::from),
        }
    }

    #[test]
    fn filter_args_all_none() {
        let args = filter_args(&query(None, None, None, None, None));
        assert!(args.is_empty());
    }

    #[test]
    fn filter_args_begin_end() {
        let args = filter_args(&query(Some("2024-01-01"), Some("2024-04-01"), None, None, None));
        assert_eq!(args, vec!["--begin", "2024-01-01", "--end", "2024-04-01"]);
    }

    #[test]
    fn filter_args_period() {
        let args = filter_args(&query(None, None, Some("monthly"), None, None));
        assert_eq!(args, vec!["--period", "monthly"]);
    }

    #[test]
    fn filter_args_depth() {
        let args = filter_args(&query(None, None, None, Some(2), None));
        assert_eq!(args, vec!["--depth", "2"]);
    }

    #[test]
    fn filter_args_account_pattern() {
        let args = filter_args(&query(None, None, None, None, Some("expenses")));
        assert_eq!(args, vec!["expenses"]);
    }

    #[test]
    fn filter_args_all_populated() {
        let args = filter_args(&query(
            Some("2024-01-01"),
            Some("2024-12-31"),
            Some("quarterly"),
            Some(3),
            Some("assets"),
        ));
        assert_eq!(
            args,
            vec![
                "--begin", "2024-01-01",
                "--end", "2024-12-31",
                "--period", "quarterly",
                "--depth", "3",
                "assets",
            ]
        );
    }

    #[test]
    fn build_args_combines_correctly() {
        let file_args = vec!["-f".to_string(), "/data/1/a.journal".to_string()];
        let filter = vec!["--begin".to_string(), "2024-01-01".to_string()];
        let args = build_args("balance", &file_args, &filter);
        assert_eq!(args, vec!["balance", "-f", "/data/1/a.journal", "--begin", "2024-01-01"]);
    }
}
