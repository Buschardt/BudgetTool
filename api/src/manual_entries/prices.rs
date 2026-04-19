use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;

use crate::auth::Claims;
use crate::core::AppState;
use crate::core::error::AppError;
use crate::core::response::ApiResponse;
use crate::manual_entries::generator::{self, PriceEntry};
use crate::manual_entries::journal::regenerate_journal_for;
use crate::manual_entries::models::{CommodityPriceInfo, CommodityPriceRecord};

#[derive(Deserialize)]
pub struct ListQuery {
    pub journal_file_id: Option<i64>,
}

#[derive(Deserialize)]
pub struct CreatePriceRequest {
    pub journal_file_id: i64,
    pub date: String,
    pub commodity: String,
    pub amount: String,
    pub target_commodity: String,
    #[serde(default)]
    pub comment: String,
}

#[derive(Deserialize)]
pub struct UpdatePriceRequest {
    pub date: Option<String>,
    pub commodity: Option<String>,
    pub amount: Option<String>,
    pub target_commodity: Option<String>,
    pub comment: Option<String>,
}

pub async fn list(
    claims: Claims,
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Result<Json<ApiResponse<Vec<CommodityPriceInfo>>>, AppError> {
    let records: Vec<CommodityPriceRecord> = if let Some(jid) = q.journal_file_id {
        sqlx::query_as(
            "SELECT id, user_id, journal_file_id, date, commodity, amount, target_commodity, comment, created_at, updated_at \
             FROM commodity_prices WHERE user_id = ? AND journal_file_id = ? ORDER BY date DESC, id DESC",
        )
        .bind(claims.sub)
        .bind(jid)
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as(
            "SELECT id, user_id, journal_file_id, date, commodity, amount, target_commodity, comment, created_at, updated_at \
             FROM commodity_prices WHERE user_id = ? ORDER BY date DESC, id DESC",
        )
        .bind(claims.sub)
        .fetch_all(&state.db)
        .await?
    };

    Ok(ApiResponse::success(
        records.into_iter().map(CommodityPriceInfo::from).collect(),
    ))
}

pub async fn create(
    claims: Claims,
    State(state): State<AppState>,
    Json(body): Json<CreatePriceRequest>,
) -> Result<Json<ApiResponse<CommodityPriceInfo>>, AppError> {
    let _: (i64,) = sqlx::query_as(
        "SELECT id FROM files WHERE id = ? AND user_id = ? AND file_type = 'journal'",
    )
    .bind(body.journal_file_id)
    .bind(claims.sub)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("journal {}", body.journal_file_id)))?;

    let entry = PriceEntry {
        date: body.date.clone(),
        commodity: body.commodity.clone(),
        amount: body.amount.clone(),
        target_commodity: body.target_commodity.clone(),
        comment: body.comment.clone(),
    };
    generator::validate_price(&entry)?;

    let record: CommodityPriceRecord = sqlx::query_as(
        "INSERT INTO commodity_prices (user_id, journal_file_id, date, commodity, amount, target_commodity, comment) \
         VALUES (?, ?, ?, ?, ?, ?, ?) \
         RETURNING id, user_id, journal_file_id, date, commodity, amount, target_commodity, comment, created_at, updated_at",
    )
    .bind(claims.sub)
    .bind(body.journal_file_id)
    .bind(&body.date)
    .bind(&body.commodity)
    .bind(&body.amount)
    .bind(&body.target_commodity)
    .bind(&body.comment)
    .fetch_one(&state.db)
    .await?;

    regenerate_journal_for(&state, body.journal_file_id).await?;

    Ok(ApiResponse::success(CommodityPriceInfo::from(record)))
}

pub async fn update(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<UpdatePriceRequest>,
) -> Result<Json<ApiResponse<CommodityPriceInfo>>, AppError> {
    let existing: Option<CommodityPriceRecord> = sqlx::query_as(
        "SELECT id, user_id, journal_file_id, date, commodity, amount, target_commodity, comment, created_at, updated_at \
         FROM commodity_prices WHERE id = ? AND user_id = ?",
    )
    .bind(id)
    .bind(claims.sub)
    .fetch_optional(&state.db)
    .await?;

    let existing = existing.ok_or_else(|| AppError::NotFound(format!("price {id}")))?;

    let new_date = body.date.unwrap_or(existing.date);
    let new_commodity = body.commodity.unwrap_or(existing.commodity);
    let new_amount = body.amount.unwrap_or(existing.amount);
    let new_target = body.target_commodity.unwrap_or(existing.target_commodity);
    let new_comment = body.comment.unwrap_or(existing.comment);

    let entry = PriceEntry {
        date: new_date.clone(),
        commodity: new_commodity.clone(),
        amount: new_amount.clone(),
        target_commodity: new_target.clone(),
        comment: new_comment.clone(),
    };
    generator::validate_price(&entry)?;

    sqlx::query(
        "UPDATE commodity_prices \
         SET date = ?, commodity = ?, amount = ?, target_commodity = ?, comment = ?, updated_at = datetime('now') \
         WHERE id = ?",
    )
    .bind(&new_date)
    .bind(&new_commodity)
    .bind(&new_amount)
    .bind(&new_target)
    .bind(&new_comment)
    .bind(id)
    .execute(&state.db)
    .await?;

    let updated: CommodityPriceRecord = sqlx::query_as(
        "SELECT id, user_id, journal_file_id, date, commodity, amount, target_commodity, comment, created_at, updated_at \
         FROM commodity_prices WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    regenerate_journal_for(&state, existing.journal_file_id).await?;

    Ok(ApiResponse::success(CommodityPriceInfo::from(updated)))
}

pub async fn delete(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<&'static str>>, AppError> {
    let existing: Option<(i64, i64)> = sqlx::query_as(
        "SELECT id, journal_file_id FROM commodity_prices WHERE id = ? AND user_id = ?",
    )
    .bind(id)
    .bind(claims.sub)
    .fetch_optional(&state.db)
    .await?;

    let (_, journal_file_id) = existing.ok_or_else(|| AppError::NotFound(format!("price {id}")))?;

    sqlx::query("DELETE FROM commodity_prices WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await?;

    regenerate_journal_for(&state, journal_file_id).await?;

    Ok(ApiResponse::success("deleted"))
}
