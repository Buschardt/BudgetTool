use axum::Json;
use axum::extract::{Multipart, Path, State};
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::Claims;
use crate::core::AppState;
use crate::core::error::AppError;
use crate::core::hledger;
use crate::core::response::ApiResponse;
use crate::files::filename::{file_extension, normalize_journal_name, sanitize_filename};
use crate::files::journal_settings::{JournalSettingsData, write_journal_with_settings};
use crate::files::models::{FileInfo, FileRecord};
use crate::rules;

const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10 MB
const ALLOWED_EXTENSIONS: &[&str] = &["journal", "csv", "rules"];

pub async fn upload(
    claims: Claims,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<FileInfo>>, AppError> {
    let field = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("multipart error: {e}")))?
        .ok_or_else(|| AppError::BadRequest("no file field in request".into()))?;

    let original_name = field
        .file_name()
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::BadRequest("missing filename in Content-Disposition".into()))?;

    let filename = sanitize_filename(&original_name)
        .ok_or_else(|| AppError::BadRequest(format!("invalid filename: {original_name}")))?;

    let ext = file_extension(&filename)
        .ok_or_else(|| AppError::BadRequest("file must have an extension".into()))?;

    if !ALLOWED_EXTENSIONS.contains(&ext) {
        return Err(AppError::BadRequest(format!(
            "unsupported file type '.{ext}'; allowed: {}",
            ALLOWED_EXTENSIONS.join(", ")
        )));
    }

    let data = field
        .bytes()
        .await
        .map_err(|e| AppError::BadRequest(format!("failed to read upload: {e}")))?;

    if data.len() > MAX_FILE_SIZE {
        return Err(AppError::PayloadTooLarge);
    }

    let user_dir = state.data_dir.join(claims.sub.to_string());
    tokio::fs::create_dir_all(&user_dir).await?;

    let disk_filename = format!("{}_{}", Uuid::new_v4(), filename);
    let disk_path = user_dir.join(&disk_filename);

    tokio::fs::write(&disk_path, &data).await?;

    let disk_path_str = disk_path.to_string_lossy().into_owned();
    let size_bytes = data.len() as i64;

    let record: FileRecord = sqlx::query_as(
        "INSERT INTO files (user_id, filename, file_type, size_bytes, disk_path) \
         VALUES (?, ?, ?, ?, ?) \
         RETURNING id, user_id, filename, file_type, size_bytes, disk_path, created_at",
    )
    .bind(claims.sub)
    .bind(&filename)
    .bind(ext)
    .bind(size_bytes)
    .bind(&disk_path_str)
    .fetch_one(&state.db)
    .await?;

    Ok(ApiResponse::success(FileInfo::from(record)))
}

pub async fn list(
    claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<FileInfo>>>, AppError> {
    let records: Vec<FileRecord> = sqlx::query_as(
        "SELECT id, user_id, filename, file_type, size_bytes, disk_path, created_at \
         FROM files WHERE user_id = ? ORDER BY created_at DESC",
    )
    .bind(claims.sub)
    .fetch_all(&state.db)
    .await?;

    Ok(ApiResponse::success(
        records.into_iter().map(FileInfo::from).collect(),
    ))
}

pub async fn get_one(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<FileInfo>>, AppError> {
    let record: Option<FileRecord> = sqlx::query_as(
        "SELECT id, user_id, filename, file_type, size_bytes, disk_path, created_at \
         FROM files WHERE id = ? AND user_id = ?",
    )
    .bind(id)
    .bind(claims.sub)
    .fetch_optional(&state.db)
    .await?;

    let record = record.ok_or_else(|| AppError::NotFound(format!("file {id}")))?;
    Ok(ApiResponse::success(FileInfo::from(record)))
}

pub async fn delete(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<&'static str>>, AppError> {
    let record: Option<FileRecord> = sqlx::query_as(
        "SELECT id, user_id, filename, file_type, size_bytes, disk_path, created_at \
         FROM files WHERE id = ? AND user_id = ?",
    )
    .bind(id)
    .bind(claims.sub)
    .fetch_optional(&state.db)
    .await?;

    let record = record.ok_or_else(|| AppError::NotFound(format!("file {id}")))?;

    match tokio::fs::remove_file(&record.disk_path).await {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(AppError::Io(e)),
    }

    sqlx::query("DELETE FROM files WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await?;

    Ok(ApiResponse::success("deleted"))
}

#[derive(Deserialize)]
pub struct ConvertRequest {
    pub rules_file_id: Option<i64>,
    pub rules_config_id: Option<i64>,
}

pub async fn convert_csv(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<ConvertRequest>,
) -> Result<Json<ApiResponse<FileInfo>>, AppError> {
    let csv: Option<FileRecord> = sqlx::query_as(
        "SELECT id, user_id, filename, file_type, size_bytes, disk_path, created_at \
         FROM files WHERE id = ? AND user_id = ?",
    )
    .bind(id)
    .bind(claims.sub)
    .fetch_optional(&state.db)
    .await?;

    let csv = csv.ok_or_else(|| AppError::NotFound(format!("file {id}")))?;

    if csv.file_type != "csv" {
        return Err(AppError::BadRequest(format!(
            "file {} is not a CSV (type: {})",
            id, csv.file_type
        )));
    }

    // Resolve rules file — prefer rules_config_id, then rules_file_id, then auto-match by stem
    let rules_disk_path: String = if let Some(config_id) = body.rules_config_id {
        rules::resolve_rules_path(&state, config_id, claims.sub).await?
    } else if let Some(rules_id) = body.rules_file_id {
        let r: Option<FileRecord> = sqlx::query_as(
            "SELECT id, user_id, filename, file_type, size_bytes, disk_path, created_at \
             FROM files WHERE id = ? AND user_id = ?",
        )
        .bind(rules_id)
        .bind(claims.sub)
        .fetch_optional(&state.db)
        .await?;

        let r = r.ok_or_else(|| AppError::NotFound(format!("rules file {rules_id}")))?;
        if r.file_type != "rules" {
            return Err(AppError::BadRequest(format!(
                "file {rules_id} is not a rules file (type: {})",
                r.file_type
            )));
        }
        r.disk_path
    } else {
        let stem = csv.filename.trim_end_matches(".csv");
        let rules_pattern = format!("{stem}.rules");
        let r: Option<FileRecord> = sqlx::query_as(
            "SELECT id, user_id, filename, file_type, size_bytes, disk_path, created_at \
             FROM files WHERE user_id = ? AND file_type = 'rules' AND filename = ? \
             ORDER BY created_at DESC LIMIT 1",
        )
        .bind(claims.sub)
        .bind(&rules_pattern)
        .fetch_optional(&state.db)
        .await?;

        r.ok_or_else(|| {
            AppError::BadRequest(format!(
                "no rules file found for '{}'; upload a .rules file, specify rules_file_id, or use a rules config",
                csv.filename
            ))
        })?.disk_path
    };

    let journal_text = hledger::run_raw(&[
        "print",
        "-f",
        &csv.disk_path,
        "--rules-file",
        &rules_disk_path,
    ])
    .await?;

    let stem = csv.filename.trim_end_matches(".csv");
    let out_filename = format!("{stem}.journal");
    let user_dir = state.data_dir.join(claims.sub.to_string());
    tokio::fs::create_dir_all(&user_dir).await?;

    let disk_filename = format!("{}_{}", Uuid::new_v4(), out_filename);
    let out_path = user_dir.join(&disk_filename);
    tokio::fs::write(&out_path, journal_text.as_bytes()).await?;

    let disk_path_str = out_path.to_string_lossy().into_owned();
    let size_bytes = journal_text.len() as i64;

    let record: FileRecord = sqlx::query_as(
        "INSERT INTO files (user_id, filename, file_type, size_bytes, disk_path) \
         VALUES (?, ?, 'journal', ?, ?) \
         RETURNING id, user_id, filename, file_type, size_bytes, disk_path, created_at",
    )
    .bind(claims.sub)
    .bind(&out_filename)
    .bind(size_bytes)
    .bind(&disk_path_str)
    .fetch_one(&state.db)
    .await?;

    Ok(ApiResponse::success(FileInfo::from(record)))
}

#[derive(Deserialize)]
pub struct CreateJournalRequest {
    pub name: String,
    pub settings: Option<JournalSettingsData>,
}

pub async fn create_journal(
    claims: Claims,
    State(state): State<AppState>,
    Json(body): Json<CreateJournalRequest>,
) -> Result<Json<ApiResponse<FileInfo>>, AppError> {
    let filename = normalize_journal_name(&body.name)?;

    let filename = sanitize_filename(&filename)
        .ok_or_else(|| AppError::BadRequest(format!("invalid journal name: {}", body.name)))?;

    let existing: Option<FileRecord> = sqlx::query_as(
        "SELECT id, user_id, filename, file_type, size_bytes, disk_path, created_at \
         FROM files WHERE user_id = ? AND filename = ? AND file_type = 'journal' LIMIT 1",
    )
    .bind(claims.sub)
    .bind(&filename)
    .fetch_optional(&state.db)
    .await?;

    if existing.is_some() {
        return Err(AppError::BadRequest(format!(
            "a journal named '{filename}' already exists"
        )));
    }

    let user_dir = state.data_dir.join(claims.sub.to_string());
    tokio::fs::create_dir_all(&user_dir).await?;

    let disk_filename = format!("{}_{}", Uuid::new_v4(), filename);
    let disk_path = user_dir.join(&disk_filename);
    tokio::fs::write(&disk_path, b"").await?;

    let disk_path_str = disk_path.to_string_lossy().into_owned();

    let record: FileRecord = sqlx::query_as(
        "INSERT INTO files (user_id, filename, file_type, size_bytes, disk_path) \
         VALUES (?, ?, 'journal', 0, ?) \
         RETURNING id, user_id, filename, file_type, size_bytes, disk_path, created_at",
    )
    .bind(claims.sub)
    .bind(&filename)
    .bind(&disk_path_str)
    .fetch_one(&state.db)
    .await?;

    let updated = write_journal_with_settings(
        &state,
        &record,
        claims.sub,
        &body.settings.unwrap_or_default(),
    )
    .await?;

    Ok(ApiResponse::success(updated))
}
