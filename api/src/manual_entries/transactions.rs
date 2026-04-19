use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;

use crate::auth::Claims;
use crate::core::AppState;
use crate::core::error::AppError;
use crate::core::response::ApiResponse;
use crate::manual_entries::generator::{self, Posting, TransactionEntry};
use crate::manual_entries::journal::regenerate_journal_for;
use crate::manual_entries::models::{ManualTransactionInfo, ManualTransactionRecord};

#[derive(Deserialize)]
pub struct ListQuery {
    pub journal_file_id: Option<i64>,
}

#[derive(Deserialize)]
pub struct CreateTransactionRequest {
    pub journal_file_id: i64,
    pub date: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub code: String,
    pub description: String,
    #[serde(default)]
    pub comment: String,
    pub postings: serde_json::Value,
}

#[derive(Deserialize)]
pub struct UpdateTransactionRequest {
    pub date: Option<String>,
    pub status: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
    pub comment: Option<String>,
    pub postings: Option<serde_json::Value>,
}

fn parse_postings(value: &serde_json::Value) -> Result<Vec<Posting>, AppError> {
    serde_json::from_value(value.clone())
        .map_err(|e| AppError::BadRequest(format!("invalid postings: {e}")))
}

pub async fn list(
    claims: Claims,
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Result<Json<ApiResponse<Vec<ManualTransactionInfo>>>, AppError> {
    let records: Vec<ManualTransactionRecord> = if let Some(jid) = q.journal_file_id {
        sqlx::query_as(
            "SELECT id, user_id, journal_file_id, date, status, code, description, comment, postings, created_at, updated_at \
             FROM manual_transactions WHERE user_id = ? AND journal_file_id = ? ORDER BY date DESC, id DESC",
        )
        .bind(claims.sub)
        .bind(jid)
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as(
            "SELECT id, user_id, journal_file_id, date, status, code, description, comment, postings, created_at, updated_at \
             FROM manual_transactions WHERE user_id = ? ORDER BY date DESC, id DESC",
        )
        .bind(claims.sub)
        .fetch_all(&state.db)
        .await?
    };

    Ok(ApiResponse::success(
        records
            .into_iter()
            .map(ManualTransactionInfo::from)
            .collect(),
    ))
}

pub async fn create(
    claims: Claims,
    State(state): State<AppState>,
    Json(body): Json<CreateTransactionRequest>,
) -> Result<Json<ApiResponse<ManualTransactionInfo>>, AppError> {
    let _: (i64,) = sqlx::query_as(
        "SELECT id FROM files WHERE id = ? AND user_id = ? AND file_type = 'journal'",
    )
    .bind(body.journal_file_id)
    .bind(claims.sub)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("journal {}", body.journal_file_id)))?;

    let postings = parse_postings(&body.postings)?;
    let entry = TransactionEntry {
        date: body.date.clone(),
        status: body.status.clone(),
        code: body.code.clone(),
        description: body.description.clone(),
        comment: body.comment.clone(),
        postings,
    };
    generator::validate_transaction(&entry)?;

    let postings_json = body.postings.to_string();

    let record: ManualTransactionRecord = sqlx::query_as(
        "INSERT INTO manual_transactions (user_id, journal_file_id, date, status, code, description, comment, postings) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?) \
         RETURNING id, user_id, journal_file_id, date, status, code, description, comment, postings, created_at, updated_at",
    )
    .bind(claims.sub)
    .bind(body.journal_file_id)
    .bind(&body.date)
    .bind(&body.status)
    .bind(&body.code)
    .bind(&body.description)
    .bind(&body.comment)
    .bind(&postings_json)
    .fetch_one(&state.db)
    .await?;

    regenerate_journal_for(&state, body.journal_file_id).await?;

    Ok(ApiResponse::success(ManualTransactionInfo::from(record)))
}

pub async fn update(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateTransactionRequest>,
) -> Result<Json<ApiResponse<ManualTransactionInfo>>, AppError> {
    let existing: Option<ManualTransactionRecord> = sqlx::query_as(
        "SELECT id, user_id, journal_file_id, date, status, code, description, comment, postings, created_at, updated_at \
         FROM manual_transactions WHERE id = ? AND user_id = ?",
    )
    .bind(id)
    .bind(claims.sub)
    .fetch_optional(&state.db)
    .await?;

    let existing = existing.ok_or_else(|| AppError::NotFound(format!("transaction {id}")))?;

    let new_date = body.date.unwrap_or(existing.date);
    let new_status = body.status.unwrap_or(existing.status);
    let new_code = body.code.unwrap_or(existing.code);
    let new_description = body.description.unwrap_or(existing.description);
    let new_comment = body.comment.unwrap_or(existing.comment);
    let new_postings_value = body
        .postings
        .unwrap_or_else(|| serde_json::from_str(&existing.postings).unwrap_or_default());

    let postings = parse_postings(&new_postings_value)?;
    let entry = TransactionEntry {
        date: new_date.clone(),
        status: new_status.clone(),
        code: new_code.clone(),
        description: new_description.clone(),
        comment: new_comment.clone(),
        postings,
    };
    generator::validate_transaction(&entry)?;

    let postings_json = new_postings_value.to_string();

    sqlx::query(
        "UPDATE manual_transactions \
         SET date = ?, status = ?, code = ?, description = ?, comment = ?, postings = ?, updated_at = datetime('now') \
         WHERE id = ?",
    )
    .bind(&new_date)
    .bind(&new_status)
    .bind(&new_code)
    .bind(&new_description)
    .bind(&new_comment)
    .bind(&postings_json)
    .bind(id)
    .execute(&state.db)
    .await?;

    let updated: ManualTransactionRecord = sqlx::query_as(
        "SELECT id, user_id, journal_file_id, date, status, code, description, comment, postings, created_at, updated_at \
         FROM manual_transactions WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    regenerate_journal_for(&state, existing.journal_file_id).await?;

    Ok(ApiResponse::success(ManualTransactionInfo::from(updated)))
}

pub async fn delete(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<&'static str>>, AppError> {
    let existing: Option<(i64, i64)> = sqlx::query_as(
        "SELECT id, journal_file_id FROM manual_transactions WHERE id = ? AND user_id = ?",
    )
    .bind(id)
    .bind(claims.sub)
    .fetch_optional(&state.db)
    .await?;

    let (_, journal_file_id) =
        existing.ok_or_else(|| AppError::NotFound(format!("transaction {id}")))?;

    sqlx::query("DELETE FROM manual_transactions WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await?;

    regenerate_journal_for(&state, journal_file_id).await?;

    Ok(ApiResponse::success("deleted"))
}
