use axum::Json;
use axum::extract::{Path, State};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppError;
use crate::manual_entries_gen::{self, PeriodicEntry, PriceEntry, TransactionEntry};
use crate::models::{
    AppState, Claims, CommodityPriceInfo, CommodityPriceRecord, ManualTransactionInfo,
    ManualTransactionRecord, PeriodicTransactionInfo, PeriodicTransactionRecord,
};
use crate::response::ApiResponse;

// ---------------------------------------------------------------------------
// Journal regeneration
// ---------------------------------------------------------------------------

pub async fn regenerate_journal(state: &AppState, user_id: i64) -> Result<(), AppError> {
    let price_records: Vec<CommodityPriceRecord> = sqlx::query_as(
        "SELECT id, user_id, date, commodity, amount, target_commodity, comment, created_at, updated_at \
         FROM commodity_prices WHERE user_id = ? ORDER BY date, id",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("db prices: {e}")))?;

    let txn_records: Vec<ManualTransactionRecord> = sqlx::query_as(
        "SELECT id, user_id, date, status, code, description, comment, postings, created_at, updated_at \
         FROM manual_transactions WHERE user_id = ? ORDER BY date, id",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("db transactions: {e}")))?;

    let periodic_records: Vec<PeriodicTransactionRecord> = sqlx::query_as(
        "SELECT id, user_id, period, description, comment, postings, created_at, updated_at \
         FROM periodic_transactions WHERE user_id = ? ORDER BY id",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("db periodics: {e}")))?;

    let prices: Vec<PriceEntry> = price_records
        .into_iter()
        .map(|r| PriceEntry {
            date: r.date,
            commodity: r.commodity,
            amount: r.amount,
            target_commodity: r.target_commodity,
            comment: r.comment,
        })
        .collect();

    let transactions: Vec<TransactionEntry> = txn_records
        .into_iter()
        .map(|r| {
            let postings = serde_json::from_str(&r.postings).unwrap_or_default();
            TransactionEntry {
                date: r.date,
                status: r.status,
                code: r.code,
                description: r.description,
                comment: r.comment,
                postings,
            }
        })
        .collect();

    let periodics: Vec<PeriodicEntry> = periodic_records
        .into_iter()
        .map(|r| {
            let postings = serde_json::from_str(&r.postings).unwrap_or_default();
            PeriodicEntry {
                period: r.period,
                description: r.description,
                comment: r.comment,
                postings,
            }
        })
        .collect();

    let journal_text =
        manual_entries_gen::generate_journal_text(&prices, &transactions, &periodics)?;

    let existing: Option<(String,)> =
        sqlx::query_as("SELECT disk_path FROM manual_entry_journals WHERE user_id = ?")
            .bind(user_id)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| AppError::Internal(format!("db manual_entry_journals: {e}")))?;

    let disk_path = if let Some((path,)) = existing {
        path
    } else {
        let user_dir = state.data_dir.join(user_id.to_string());
        tokio::fs::create_dir_all(&user_dir).await?;
        let filename = format!("{}_manual-entries.journal", Uuid::new_v4());
        let path = user_dir.join(&filename);
        let path_str = path.to_string_lossy().into_owned();
        sqlx::query("INSERT INTO manual_entry_journals (user_id, disk_path) VALUES (?, ?)")
            .bind(user_id)
            .bind(&path_str)
            .execute(&state.db)
            .await
            .map_err(|e| AppError::Internal(format!("db insert journal: {e}")))?;
        path_str
    };

    tokio::fs::write(&disk_path, journal_text.as_bytes()).await?;

    sqlx::query("UPDATE manual_entry_journals SET updated_at = datetime('now') WHERE user_id = ?")
        .bind(user_id)
        .execute(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("db update journal ts: {e}")))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Prices
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct CreatePriceRequest {
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

pub async fn list_prices(
    claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<CommodityPriceInfo>>>, AppError> {
    let records: Vec<CommodityPriceRecord> = sqlx::query_as(
        "SELECT id, user_id, date, commodity, amount, target_commodity, comment, created_at, updated_at \
         FROM commodity_prices WHERE user_id = ? ORDER BY date DESC, id DESC",
    )
    .bind(claims.sub)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("db: {e}")))?;

    Ok(ApiResponse::success(
        records.into_iter().map(CommodityPriceInfo::from).collect(),
    ))
}

pub async fn create_price(
    claims: Claims,
    State(state): State<AppState>,
    Json(body): Json<CreatePriceRequest>,
) -> Result<Json<ApiResponse<CommodityPriceInfo>>, AppError> {
    let entry = PriceEntry {
        date: body.date.clone(),
        commodity: body.commodity.clone(),
        amount: body.amount.clone(),
        target_commodity: body.target_commodity.clone(),
        comment: body.comment.clone(),
    };
    manual_entries_gen::validate_price(&entry)?;

    let record: CommodityPriceRecord = sqlx::query_as(
        "INSERT INTO commodity_prices (user_id, date, commodity, amount, target_commodity, comment) \
         VALUES (?, ?, ?, ?, ?, ?) \
         RETURNING id, user_id, date, commodity, amount, target_commodity, comment, created_at, updated_at",
    )
    .bind(claims.sub)
    .bind(&body.date)
    .bind(&body.commodity)
    .bind(&body.amount)
    .bind(&body.target_commodity)
    .bind(&body.comment)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("db insert: {e}")))?;

    regenerate_journal(&state, claims.sub).await?;

    Ok(ApiResponse::success(CommodityPriceInfo::from(record)))
}

pub async fn update_price(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<UpdatePriceRequest>,
) -> Result<Json<ApiResponse<CommodityPriceInfo>>, AppError> {
    let existing: Option<CommodityPriceRecord> = sqlx::query_as(
        "SELECT id, user_id, date, commodity, amount, target_commodity, comment, created_at, updated_at \
         FROM commodity_prices WHERE id = ? AND user_id = ?",
    )
    .bind(id)
    .bind(claims.sub)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("db: {e}")))?;

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
    manual_entries_gen::validate_price(&entry)?;

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
    .await
    .map_err(|e| AppError::Internal(format!("db update: {e}")))?;

    let updated: CommodityPriceRecord = sqlx::query_as(
        "SELECT id, user_id, date, commodity, amount, target_commodity, comment, created_at, updated_at \
         FROM commodity_prices WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("db: {e}")))?;

    regenerate_journal(&state, claims.sub).await?;

    Ok(ApiResponse::success(CommodityPriceInfo::from(updated)))
}

pub async fn delete_price(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<&'static str>>, AppError> {
    let existing: Option<(i64,)> =
        sqlx::query_as("SELECT id FROM commodity_prices WHERE id = ? AND user_id = ?")
            .bind(id)
            .bind(claims.sub)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

    existing.ok_or_else(|| AppError::NotFound(format!("price {id}")))?;

    sqlx::query("DELETE FROM commodity_prices WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("db delete: {e}")))?;

    regenerate_journal(&state, claims.sub).await?;

    Ok(ApiResponse::success("deleted"))
}

// ---------------------------------------------------------------------------
// Manual transactions
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct CreateTransactionRequest {
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

fn parse_postings(value: &serde_json::Value) -> Result<Vec<manual_entries_gen::Posting>, AppError> {
    serde_json::from_value(value.clone())
        .map_err(|e| AppError::BadRequest(format!("invalid postings: {e}")))
}

pub async fn list_transactions(
    claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<ManualTransactionInfo>>>, AppError> {
    let records: Vec<ManualTransactionRecord> = sqlx::query_as(
        "SELECT id, user_id, date, status, code, description, comment, postings, created_at, updated_at \
         FROM manual_transactions WHERE user_id = ? ORDER BY date DESC, id DESC",
    )
    .bind(claims.sub)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("db: {e}")))?;

    Ok(ApiResponse::success(
        records
            .into_iter()
            .map(ManualTransactionInfo::from)
            .collect(),
    ))
}

pub async fn create_transaction(
    claims: Claims,
    State(state): State<AppState>,
    Json(body): Json<CreateTransactionRequest>,
) -> Result<Json<ApiResponse<ManualTransactionInfo>>, AppError> {
    let postings = parse_postings(&body.postings)?;
    let entry = TransactionEntry {
        date: body.date.clone(),
        status: body.status.clone(),
        code: body.code.clone(),
        description: body.description.clone(),
        comment: body.comment.clone(),
        postings,
    };
    manual_entries_gen::validate_transaction(&entry)?;

    let postings_json = body.postings.to_string();

    let record: ManualTransactionRecord = sqlx::query_as(
        "INSERT INTO manual_transactions (user_id, date, status, code, description, comment, postings) \
         VALUES (?, ?, ?, ?, ?, ?, ?) \
         RETURNING id, user_id, date, status, code, description, comment, postings, created_at, updated_at",
    )
    .bind(claims.sub)
    .bind(&body.date)
    .bind(&body.status)
    .bind(&body.code)
    .bind(&body.description)
    .bind(&body.comment)
    .bind(&postings_json)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("db insert: {e}")))?;

    regenerate_journal(&state, claims.sub).await?;

    Ok(ApiResponse::success(ManualTransactionInfo::from(record)))
}

pub async fn update_transaction(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateTransactionRequest>,
) -> Result<Json<ApiResponse<ManualTransactionInfo>>, AppError> {
    let existing: Option<ManualTransactionRecord> = sqlx::query_as(
        "SELECT id, user_id, date, status, code, description, comment, postings, created_at, updated_at \
         FROM manual_transactions WHERE id = ? AND user_id = ?",
    )
    .bind(id)
    .bind(claims.sub)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("db: {e}")))?;

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
    manual_entries_gen::validate_transaction(&entry)?;

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
    .await
    .map_err(|e| AppError::Internal(format!("db update: {e}")))?;

    let updated: ManualTransactionRecord = sqlx::query_as(
        "SELECT id, user_id, date, status, code, description, comment, postings, created_at, updated_at \
         FROM manual_transactions WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("db: {e}")))?;

    regenerate_journal(&state, claims.sub).await?;

    Ok(ApiResponse::success(ManualTransactionInfo::from(updated)))
}

pub async fn delete_transaction(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<&'static str>>, AppError> {
    let existing: Option<(i64,)> =
        sqlx::query_as("SELECT id FROM manual_transactions WHERE id = ? AND user_id = ?")
            .bind(id)
            .bind(claims.sub)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

    existing.ok_or_else(|| AppError::NotFound(format!("transaction {id}")))?;

    sqlx::query("DELETE FROM manual_transactions WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("db delete: {e}")))?;

    regenerate_journal(&state, claims.sub).await?;

    Ok(ApiResponse::success("deleted"))
}

// ---------------------------------------------------------------------------
// Periodic transactions
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct CreatePeriodicRequest {
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

pub async fn list_periodics(
    claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<PeriodicTransactionInfo>>>, AppError> {
    let records: Vec<PeriodicTransactionRecord> = sqlx::query_as(
        "SELECT id, user_id, period, description, comment, postings, created_at, updated_at \
         FROM periodic_transactions WHERE user_id = ? ORDER BY id",
    )
    .bind(claims.sub)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("db: {e}")))?;

    Ok(ApiResponse::success(
        records
            .into_iter()
            .map(PeriodicTransactionInfo::from)
            .collect(),
    ))
}

pub async fn create_periodic(
    claims: Claims,
    State(state): State<AppState>,
    Json(body): Json<CreatePeriodicRequest>,
) -> Result<Json<ApiResponse<PeriodicTransactionInfo>>, AppError> {
    let postings = parse_postings(&body.postings)?;
    let entry = PeriodicEntry {
        period: body.period.clone(),
        description: body.description.clone(),
        comment: body.comment.clone(),
        postings,
    };
    manual_entries_gen::validate_periodic(&entry)?;

    let postings_json = body.postings.to_string();

    let record: PeriodicTransactionRecord = sqlx::query_as(
        "INSERT INTO periodic_transactions (user_id, period, description, comment, postings) \
         VALUES (?, ?, ?, ?, ?) \
         RETURNING id, user_id, period, description, comment, postings, created_at, updated_at",
    )
    .bind(claims.sub)
    .bind(&body.period)
    .bind(&body.description)
    .bind(&body.comment)
    .bind(&postings_json)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("db insert: {e}")))?;

    regenerate_journal(&state, claims.sub).await?;

    Ok(ApiResponse::success(PeriodicTransactionInfo::from(record)))
}

pub async fn update_periodic(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<UpdatePeriodicRequest>,
) -> Result<Json<ApiResponse<PeriodicTransactionInfo>>, AppError> {
    let existing: Option<PeriodicTransactionRecord> = sqlx::query_as(
        "SELECT id, user_id, period, description, comment, postings, created_at, updated_at \
         FROM periodic_transactions WHERE id = ? AND user_id = ?",
    )
    .bind(id)
    .bind(claims.sub)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("db: {e}")))?;

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
    manual_entries_gen::validate_periodic(&entry)?;

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
    .await
    .map_err(|e| AppError::Internal(format!("db update: {e}")))?;

    let updated: PeriodicTransactionRecord = sqlx::query_as(
        "SELECT id, user_id, period, description, comment, postings, created_at, updated_at \
         FROM periodic_transactions WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("db: {e}")))?;

    regenerate_journal(&state, claims.sub).await?;

    Ok(ApiResponse::success(PeriodicTransactionInfo::from(updated)))
}

pub async fn delete_periodic(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<&'static str>>, AppError> {
    let existing: Option<(i64,)> =
        sqlx::query_as("SELECT id FROM periodic_transactions WHERE id = ? AND user_id = ?")
            .bind(id)
            .bind(claims.sub)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

    existing.ok_or_else(|| AppError::NotFound(format!("periodic {id}")))?;

    sqlx::query("DELETE FROM periodic_transactions WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("db delete: {e}")))?;

    regenerate_journal(&state, claims.sub).await?;

    Ok(ApiResponse::success("deleted"))
}
