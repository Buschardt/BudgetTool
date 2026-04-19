use axum::Json;
use axum::extract::{Path, State};
use serde::Deserialize;

use crate::auth::Claims;
use crate::core::AppState;
use crate::core::error::AppError;
use crate::core::hledger;
use crate::core::response::ApiResponse;
use crate::rules::generator::{self, RulesConfig};
use crate::rules::models::{RulesConfigDetail, RulesConfigInfo, RulesConfigRecord};
use crate::rules::service::generate_and_write;

#[derive(Deserialize)]
pub struct CreateRequest {
    pub name: String,
    pub description: Option<String>,
    pub config: serde_json::Value,
}

#[derive(Deserialize)]
pub struct UpdateRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub config: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct PreviewRequest {
    pub csv_file_id: i64,
}

fn parse_config(value: &serde_json::Value) -> Result<RulesConfig, AppError> {
    serde_json::from_value(value.clone())
        .map_err(|e| AppError::BadRequest(format!("invalid rules config: {e}")))
}

pub async fn list(
    claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<RulesConfigInfo>>>, AppError> {
    let records: Vec<RulesConfigRecord> = sqlx::query_as(
        "SELECT id, user_id, name, description, config, disk_path, created_at, updated_at \
         FROM rules_configs WHERE user_id = ? ORDER BY updated_at DESC",
    )
    .bind(claims.sub)
    .fetch_all(&state.db)
    .await?;

    Ok(ApiResponse::success(
        records.into_iter().map(RulesConfigInfo::from).collect(),
    ))
}

pub async fn get_one(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<RulesConfigDetail>>, AppError> {
    let record: Option<RulesConfigRecord> = sqlx::query_as(
        "SELECT id, user_id, name, description, config, disk_path, created_at, updated_at \
         FROM rules_configs WHERE id = ? AND user_id = ?",
    )
    .bind(id)
    .bind(claims.sub)
    .fetch_optional(&state.db)
    .await?;

    let record = record.ok_or_else(|| AppError::NotFound(format!("rules config {id}")))?;
    Ok(ApiResponse::success(RulesConfigDetail::from(record)))
}

pub async fn create(
    claims: Claims,
    State(state): State<AppState>,
    Json(body): Json<CreateRequest>,
) -> Result<Json<ApiResponse<RulesConfigDetail>>, AppError> {
    if body.name.trim().is_empty() {
        return Err(AppError::BadRequest("name is required".into()));
    }

    let config: RulesConfig = parse_config(&body.config)?;
    generator::validate(&config)?;

    let config_json = serde_json::to_string(&config)
        .map_err(|e| AppError::Internal(format!("serialize config: {e}")))?;

    let description = body.description.unwrap_or_default();

    let record: RulesConfigRecord = sqlx::query_as(
        "INSERT INTO rules_configs (user_id, name, description, config) \
         VALUES (?, ?, ?, ?) \
         RETURNING id, user_id, name, description, config, disk_path, created_at, updated_at",
    )
    .bind(claims.sub)
    .bind(&body.name)
    .bind(&description)
    .bind(&config_json)
    .fetch_one(&state.db)
    .await?;

    generate_and_write(&state, record.id, claims.sub, &config, &body.name).await?;

    let updated: RulesConfigRecord = sqlx::query_as(
        "SELECT id, user_id, name, description, config, disk_path, created_at, updated_at \
         FROM rules_configs WHERE id = ?",
    )
    .bind(record.id)
    .fetch_one(&state.db)
    .await?;

    Ok(ApiResponse::success(RulesConfigDetail::from(updated)))
}

pub async fn update(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateRequest>,
) -> Result<Json<ApiResponse<RulesConfigDetail>>, AppError> {
    let existing: Option<RulesConfigRecord> = sqlx::query_as(
        "SELECT id, user_id, name, description, config, disk_path, created_at, updated_at \
         FROM rules_configs WHERE id = ? AND user_id = ?",
    )
    .bind(id)
    .bind(claims.sub)
    .fetch_optional(&state.db)
    .await?;

    let existing = existing.ok_or_else(|| AppError::NotFound(format!("rules config {id}")))?;

    let new_name = body.name.unwrap_or(existing.name.clone());
    let new_description = body.description.unwrap_or(existing.description.clone());

    let new_config: RulesConfig = match body.config {
        Some(ref v) => parse_config(v)?,
        None => serde_json::from_str(&existing.config)
            .map_err(|e| AppError::Internal(format!("parse stored config: {e}")))?,
    };
    generator::validate(&new_config)?;

    let config_json = serde_json::to_string(&new_config)
        .map_err(|e| AppError::Internal(format!("serialize config: {e}")))?;

    sqlx::query(
        "UPDATE rules_configs \
         SET name = ?, description = ?, config = ?, updated_at = datetime('now') \
         WHERE id = ?",
    )
    .bind(&new_name)
    .bind(&new_description)
    .bind(&config_json)
    .bind(id)
    .execute(&state.db)
    .await?;

    if let Some(ref old_path) = existing.disk_path {
        let _ = tokio::fs::remove_file(old_path).await;
    }

    generate_and_write(&state, id, claims.sub, &new_config, &new_name).await?;

    let updated: RulesConfigRecord = sqlx::query_as(
        "SELECT id, user_id, name, description, config, disk_path, created_at, updated_at \
         FROM rules_configs WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    Ok(ApiResponse::success(RulesConfigDetail::from(updated)))
}

pub async fn delete(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<&'static str>>, AppError> {
    let record: Option<RulesConfigRecord> = sqlx::query_as(
        "SELECT id, user_id, name, description, config, disk_path, created_at, updated_at \
         FROM rules_configs WHERE id = ? AND user_id = ?",
    )
    .bind(id)
    .bind(claims.sub)
    .fetch_optional(&state.db)
    .await?;

    let record = record.ok_or_else(|| AppError::NotFound(format!("rules config {id}")))?;

    if let Some(ref path) = record.disk_path {
        match tokio::fs::remove_file(path).await {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(AppError::Io(e)),
        }
    }

    sqlx::query("DELETE FROM rules_configs WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await?;

    Ok(ApiResponse::success("deleted"))
}

pub async fn preview(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<PreviewRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    let record: Option<RulesConfigRecord> = sqlx::query_as(
        "SELECT id, user_id, name, description, config, disk_path, created_at, updated_at \
         FROM rules_configs WHERE id = ? AND user_id = ?",
    )
    .bind(id)
    .bind(claims.sub)
    .fetch_optional(&state.db)
    .await?;

    let record = record.ok_or_else(|| AppError::NotFound(format!("rules config {id}")))?;

    let csv_path: Option<(String, String)> =
        sqlx::query_as("SELECT disk_path, file_type FROM files WHERE id = ? AND user_id = ?")
            .bind(body.csv_file_id)
            .bind(claims.sub)
            .fetch_optional(&state.db)
            .await?;

    let (csv_disk_path, file_type) =
        csv_path.ok_or_else(|| AppError::NotFound(format!("file {}", body.csv_file_id)))?;

    if file_type != "csv" {
        return Err(AppError::BadRequest(format!(
            "file {} is not a CSV",
            body.csv_file_id
        )));
    }

    let rules_path = match record.disk_path {
        Some(ref p) => p.clone(),
        None => {
            let config: RulesConfig = serde_json::from_str(&record.config)
                .map_err(|e| AppError::Internal(format!("parse config: {e}")))?;
            generate_and_write(&state, record.id, claims.sub, &config, &record.name).await?
        }
    };

    let journal_text =
        hledger::run_raw(&["print", "-f", &csv_disk_path, "--rules-file", &rules_path]).await?;

    Ok(ApiResponse::success(journal_text))
}
