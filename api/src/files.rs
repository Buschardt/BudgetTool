use axum::Json;
use axum::extract::{Multipart, Path, State};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppError;
use crate::hledger;
use crate::models::{AppState, Claims, FileInfo, FileRecord};
use crate::response::ApiResponse;

const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10 MB
const ALLOWED_EXTENSIONS: &[&str] = &["journal", "csv", "rules"];

/// Sanitize an uploaded filename: keep only the final path component, reject traversal.
/// Returns None if the result is empty or a bare dot-segment.
fn sanitize_filename(name: &str) -> Option<String> {
    // Strip path separators and take only the last segment
    let basename = name
        .replace('\\', "/")
        .split('/')
        .rfind(|s| !s.is_empty())
        .map(|s| s.to_string())?;

    // Reject dot-only segments and names containing null bytes
    if basename == "." || basename == ".." || basename.contains('\0') {
        return None;
    }

    Some(basename)
}

/// Extract the file extension from a filename.
/// Returns None for dotfiles (e.g. `.hidden`) and files with no extension.
fn file_extension(filename: &str) -> Option<&str> {
    // Find the last dot that isn't the very first character
    let dot_pos = filename[1..].rfind('.')?.checked_add(1)?;
    let ext = &filename[dot_pos + 1..];
    if ext.is_empty() { None } else { Some(ext) }
}

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
    .await
    .map_err(|e| AppError::Internal(format!("db insert: {e}")))?;

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
    .await
    .map_err(|e| AppError::Internal(format!("db: {e}")))?;

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
    .await
    .map_err(|e| AppError::Internal(format!("db: {e}")))?;

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
    .await
    .map_err(|e| AppError::Internal(format!("db: {e}")))?;

    let record = record.ok_or_else(|| AppError::NotFound(format!("file {id}")))?;

    match tokio::fs::remove_file(&record.disk_path).await {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(AppError::Io(e)),
    }

    sqlx::query("DELETE FROM files WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("db delete: {e}")))?;

    Ok(ApiResponse::success("deleted"))
}

#[derive(Deserialize)]
pub struct ConvertRequest {
    pub rules_file_id: Option<i64>,
}

pub async fn convert_csv(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<ConvertRequest>,
) -> Result<Json<ApiResponse<FileInfo>>, AppError> {
    // Fetch the CSV file
    let csv: Option<FileRecord> = sqlx::query_as(
        "SELECT id, user_id, filename, file_type, size_bytes, disk_path, created_at \
         FROM files WHERE id = ? AND user_id = ?",
    )
    .bind(id)
    .bind(claims.sub)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("db: {e}")))?;

    let csv = csv.ok_or_else(|| AppError::NotFound(format!("file {id}")))?;

    if csv.file_type != "csv" {
        return Err(AppError::BadRequest(format!(
            "file {} is not a CSV (type: {})",
            id, csv.file_type
        )));
    }

    // Resolve rules file
    let rules = if let Some(rules_id) = body.rules_file_id {
        let r: Option<FileRecord> = sqlx::query_as(
            "SELECT id, user_id, filename, file_type, size_bytes, disk_path, created_at \
             FROM files WHERE id = ? AND user_id = ?",
        )
        .bind(rules_id)
        .bind(claims.sub)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let r = r.ok_or_else(|| AppError::NotFound(format!("rules file {rules_id}")))?;
        if r.file_type != "rules" {
            return Err(AppError::BadRequest(format!(
                "file {rules_id} is not a rules file (type: {})",
                r.file_type
            )));
        }
        r
    } else {
        // Auto-match: look for a rules file with the same stem as the CSV
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
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        r.ok_or_else(|| {
            AppError::BadRequest(format!(
                "no rules file found for '{}'; upload a .rules file or specify rules_file_id",
                csv.filename
            ))
        })?
    };

    // Run hledger conversion
    let journal_text = hledger::run_raw(&[
        "print",
        "-f",
        &csv.disk_path,
        "--rules-file",
        &rules.disk_path,
    ])
    .await?;

    // Write output journal file
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
    .await
    .map_err(|e| AppError::Internal(format!("db insert: {e}")))?;

    Ok(ApiResponse::success(FileInfo::from(record)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_normal_filename() {
        assert_eq!(
            sanitize_filename("transactions.csv"),
            Some("transactions.csv".into())
        );
    }

    #[test]
    fn sanitize_strips_path_separators() {
        assert_eq!(sanitize_filename("../../etc/passwd"), Some("passwd".into()));
        assert_eq!(
            sanitize_filename("foo/bar/baz.journal"),
            Some("baz.journal".into())
        );
    }

    #[test]
    fn sanitize_strips_windows_separators() {
        assert_eq!(
            sanitize_filename("C:\\Users\\bob\\file.csv"),
            Some("file.csv".into())
        );
    }

    #[test]
    fn sanitize_rejects_bare_dotdot() {
        assert_eq!(sanitize_filename(".."), None);
        assert_eq!(sanitize_filename("."), None);
    }

    #[test]
    fn sanitize_rejects_null_byte() {
        assert_eq!(sanitize_filename("file\0name.csv"), None);
    }

    #[test]
    fn sanitize_rejects_empty() {
        assert_eq!(sanitize_filename(""), None);
        assert_eq!(sanitize_filename("///"), None);
    }

    #[test]
    fn extension_detection() {
        assert_eq!(file_extension("foo.csv"), Some("csv"));
        assert_eq!(file_extension("foo.journal"), Some("journal"));
        assert_eq!(file_extension("foo.rules"), Some("rules"));
        assert_eq!(file_extension("noext"), None);
        assert_eq!(file_extension(".hidden"), None);
    }
}
