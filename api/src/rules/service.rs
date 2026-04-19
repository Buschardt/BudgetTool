use uuid::Uuid;

use crate::core::AppState;
use crate::core::error::AppError;
use crate::rules::generator::{self, RulesConfig};
use crate::rules::models::RulesConfigRecord;

/// Resolve include paths for the given include IDs, scoped to the user.
pub async fn resolve_include_paths(
    db: &sqlx::SqlitePool,
    user_id: i64,
    ids: &[i64],
) -> Result<Vec<(i64, String)>, AppError> {
    let mut paths = Vec::new();
    for &id in ids {
        let row: Option<(i64, Option<String>)> =
            sqlx::query_as("SELECT id, disk_path FROM rules_configs WHERE id = ? AND user_id = ?")
                .bind(id)
                .bind(user_id)
                .fetch_optional(db)
                .await?;

        let (_, disk_path) = row
            .ok_or_else(|| AppError::BadRequest(format!("included rules config {id} not found")))?;

        let path = disk_path.ok_or_else(|| {
            AppError::BadRequest(format!(
                "included rules config {id} has not been generated yet"
            ))
        })?;
        paths.push((id, path));
    }
    Ok(paths)
}

/// Generate and write the `.rules` file for the given config record.
/// Returns the disk path of the generated file.
pub async fn generate_and_write(
    state: &AppState,
    record_id: i64,
    user_id: i64,
    config: &RulesConfig,
    name: &str,
) -> Result<String, AppError> {
    let include_paths = resolve_include_paths(&state.db, user_id, &config.includes).await?;
    let rules_text = generator::generate_rules_text(config, &include_paths)?;

    let user_dir = state.data_dir.join(user_id.to_string());
    tokio::fs::create_dir_all(&user_dir).await?;

    let safe_name = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();
    let filename = format!("{}_{}.rules", Uuid::new_v4(), safe_name);
    let path = user_dir.join(&filename);

    tokio::fs::write(&path, rules_text.as_bytes()).await?;

    sqlx::query(
        "UPDATE rules_configs SET disk_path = ?, updated_at = datetime('now') WHERE id = ?",
    )
    .bind(path.to_string_lossy().as_ref())
    .bind(record_id)
    .execute(&state.db)
    .await?;

    Ok(path.to_string_lossy().into_owned())
}

/// Resolve a rules config ID to its disk path, generating the file if needed.
/// Called from files::convert_csv.
pub async fn resolve_rules_path(
    state: &AppState,
    config_id: i64,
    user_id: i64,
) -> Result<String, AppError> {
    let record: Option<RulesConfigRecord> = sqlx::query_as(
        "SELECT id, user_id, name, description, config, disk_path, created_at, updated_at \
         FROM rules_configs WHERE id = ? AND user_id = ?",
    )
    .bind(config_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?;

    let record = record.ok_or_else(|| AppError::NotFound(format!("rules config {config_id}")))?;

    match record.disk_path {
        Some(p) => Ok(p),
        None => {
            let config: RulesConfig = serde_json::from_str(&record.config)
                .map_err(|e| AppError::Internal(format!("parse config: {e}")))?;
            generate_and_write(state, record.id, user_id, &config, &record.name).await
        }
    }
}
