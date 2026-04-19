use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;

use crate::auth::Claims;
use crate::core::AppState;
use crate::core::error::AppError;
use crate::core::response::ApiResponse;
use crate::manual_entries::generator::{self, PeriodicEntry, Posting};
use crate::manual_entries::journal::regenerate_journal_for;
use crate::manual_entries::models::{PeriodicTransactionInfo, PeriodicTransactionRecord};

#[derive(Deserialize)]
pub struct ListQuery {
    pub journal_file_id: Option<i64>,
}

#[derive(Deserialize)]
pub struct CreatePeriodicRequest {
    pub journal_file_id: i64,
    pub period: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub comment: String,
    pub postings: serde_json::Value,
}

#[derive(Deserialize)]
pub struct UpdatePeriodicRequest {
    pub period: Option<String>,
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
) -> Result<Json<ApiResponse<Vec<PeriodicTransactionInfo>>>, AppError> {
    let records: Vec<PeriodicTransactionRecord> = if let Some(jid) = q.journal_file_id {
        sqlx::query_as(
            "SELECT id, user_id, journal_file_id, period, description, comment, postings, created_at, updated_at \
             FROM periodic_transactions WHERE user_id = ? AND journal_file_id = ? ORDER BY id",
        )
        .bind(claims.sub)
        .bind(jid)
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as(
            "SELECT id, user_id, journal_file_id, period, description, comment, postings, created_at, updated_at \
             FROM periodic_transactions WHERE user_id = ? ORDER BY id",
        )
        .bind(claims.sub)
        .fetch_all(&state.db)
        .await?
    };

    Ok(ApiResponse::success(
        records
            .into_iter()
            .map(PeriodicTransactionInfo::from)
            .collect(),
    ))
}

pub async fn create(
    claims: Claims,
    State(state): State<AppState>,
    Json(body): Json<CreatePeriodicRequest>,
) -> Result<Json<ApiResponse<PeriodicTransactionInfo>>, AppError> {
    let _: (i64,) = sqlx::query_as(
        "SELECT id FROM files WHERE id = ? AND user_id = ? AND file_type = 'journal'",
    )
    .bind(body.journal_file_id)
    .bind(claims.sub)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("journal {}", body.journal_file_id)))?;

    let postings = parse_postings(&body.postings)?;
    let entry = PeriodicEntry {
        period: body.period.clone(),
        description: body.description.clone(),
        comment: body.comment.clone(),
        postings,
    };
    generator::validate_periodic(&entry)?;

    let postings_json = body.postings.to_string();

    let record: PeriodicTransactionRecord = sqlx::query_as(
        "INSERT INTO periodic_transactions (user_id, journal_file_id, period, description, comment, postings) \
         VALUES (?, ?, ?, ?, ?, ?) \
         RETURNING id, user_id, journal_file_id, period, description, comment, postings, created_at, updated_at",
    )
    .bind(claims.sub)
    .bind(body.journal_file_id)
    .bind(&body.period)
    .bind(&body.description)
    .bind(&body.comment)
    .bind(&postings_json)
    .fetch_one(&state.db)
    .await?;

    regenerate_journal_for(&state, body.journal_file_id).await?;

    Ok(ApiResponse::success(PeriodicTransactionInfo::from(record)))
}

pub async fn update(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<UpdatePeriodicRequest>,
) -> Result<Json<ApiResponse<PeriodicTransactionInfo>>, AppError> {
    let existing: Option<PeriodicTransactionRecord> = sqlx::query_as(
        "SELECT id, user_id, journal_file_id, period, description, comment, postings, created_at, updated_at \
         FROM periodic_transactions WHERE id = ? AND user_id = ?",
    )
    .bind(id)
    .bind(claims.sub)
    .fetch_optional(&state.db)
    .await?;

    let existing = existing.ok_or_else(|| AppError::NotFound(format!("periodic {id}")))?;

    let new_period = body.period.unwrap_or(existing.period);
    let new_description = body.description.unwrap_or(existing.description);
    let new_comment = body.comment.unwrap_or(existing.comment);
    let new_postings_value = body
        .postings
        .unwrap_or_else(|| serde_json::from_str(&existing.postings).unwrap_or_default());

    let postings = parse_postings(&new_postings_value)?;
    let entry = PeriodicEntry {
        period: new_period.clone(),
        description: new_description.clone(),
        comment: new_comment.clone(),
        postings,
    };
    generator::validate_periodic(&entry)?;

    let postings_json = new_postings_value.to_string();

    sqlx::query(
        "UPDATE periodic_transactions \
         SET period = ?, description = ?, comment = ?, postings = ?, updated_at = datetime('now') \
         WHERE id = ?",
    )
    .bind(&new_period)
    .bind(&new_description)
    .bind(&new_comment)
    .bind(&postings_json)
    .bind(id)
    .execute(&state.db)
    .await?;

    let updated: PeriodicTransactionRecord = sqlx::query_as(
        "SELECT id, user_id, journal_file_id, period, description, comment, postings, created_at, updated_at \
         FROM periodic_transactions WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    regenerate_journal_for(&state, existing.journal_file_id).await?;

    Ok(ApiResponse::success(PeriodicTransactionInfo::from(updated)))
}

pub async fn delete(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<&'static str>>, AppError> {
    let existing: Option<(i64, i64)> = sqlx::query_as(
        "SELECT id, journal_file_id FROM periodic_transactions WHERE id = ? AND user_id = ?",
    )
    .bind(id)
    .bind(claims.sub)
    .fetch_optional(&state.db)
    .await?;

    let (_, journal_file_id) =
        existing.ok_or_else(|| AppError::NotFound(format!("periodic {id}")))?;

    sqlx::query("DELETE FROM periodic_transactions WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await?;

    regenerate_journal_for(&state, journal_file_id).await?;

    Ok(ApiResponse::success("deleted"))
}
