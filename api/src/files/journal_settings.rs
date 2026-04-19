use std::collections::{HashMap, HashSet};

use axum::Json;
use axum::extract::{Path, State};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::auth::Claims;
use crate::core::AppState;
use crate::core::error::AppError;
use crate::core::response::ApiResponse;
use crate::files::models::{FileInfo, FileRecord};

const HEADER_BEGIN: &str = "; BEGIN BudgetTool managed header";
const HEADER_END: &str = "; END BudgetTool managed header";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CommoditySetting {
    pub sample: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AccountSetting {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct JournalSettingsData {
    #[serde(default)]
    pub default_commodity: Option<String>,
    #[serde(default)]
    pub decimal_mark: Option<String>,
    #[serde(default)]
    pub commodities: Vec<CommoditySetting>,
    #[serde(default)]
    pub accounts: Vec<AccountSetting>,
    #[serde(default)]
    pub includes: Vec<i64>,
}

#[derive(Debug, Serialize)]
pub struct JournalSettingsDetail {
    pub file: FileInfo,
    pub settings: JournalSettingsData,
}

#[derive(Debug, sqlx::FromRow)]
struct JournalSettingsRow {
    file_id: i64,
    default_commodity: Option<String>,
    decimal_mark: Option<String>,
    commodities_json: String,
    accounts_json: String,
    includes_json: String,
}

#[derive(Debug, Clone)]
struct JournalFileRef {
    id: i64,
    disk_path: String,
}

#[derive(Debug)]
struct ImportedJournal {
    settings: JournalSettingsData,
    unsupported_preamble: String,
    body: String,
    has_managed_header: bool,
}

#[derive(Debug)]
enum PreambleLine {
    Unsupported(String),
    DefaultCommodity { value: String },
    DecimalMark { value: String },
    Commodity { sample: String },
    Account { name: String },
    Include { path: String, raw: String },
}

#[derive(Debug, Deserialize)]
pub struct UpdateJournalSettingsRequest {
    pub settings: JournalSettingsData,
}

fn parse_json_vec<T>(raw: &str, name: &str) -> Result<Vec<T>, AppError>
where
    T: for<'de> Deserialize<'de>,
{
    serde_json::from_str(raw).map_err(|e| AppError::Internal(format!("parse stored {name}: {e}")))
}

fn map_row_to_data(row: JournalSettingsRow) -> Result<JournalSettingsData, AppError> {
    let _ = row.file_id;
    Ok(JournalSettingsData {
        default_commodity: row.default_commodity.filter(|s| !s.trim().is_empty()),
        decimal_mark: row.decimal_mark.filter(|s| !s.trim().is_empty()),
        commodities: parse_json_vec(&row.commodities_json, "commodities")?,
        accounts: parse_json_vec(&row.accounts_json, "accounts")?,
        includes: parse_json_vec(&row.includes_json, "includes")?,
    })
}

fn transaction_start_regex() -> Regex {
    Regex::new(r"^(?:\d{4}[-/]\d{1,2}[-/]\d{1,2}|[~=])").unwrap()
}

fn split_inclusive_lines(text: &str) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }

    let mut lines: Vec<String> = text.split_inclusive('\n').map(String::from).collect();
    if !text.ends_with('\n')
        && let Some(last) = lines.last_mut()
        && last.ends_with('\n')
    {
        last.pop();
    }
    lines
}

fn parse_preamble_line(line: &str) -> PreambleLine {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with('#') {
        return PreambleLine::Unsupported(line.to_string());
    }

    if let Some(rest) = trimmed.strip_prefix("decimal-mark") {
        let value = rest.trim().to_string();
        return if value.is_empty() {
            PreambleLine::Unsupported(line.to_string())
        } else {
            PreambleLine::DecimalMark { value }
        };
    }

    if let Some(rest) = trimmed.strip_prefix("commodity") {
        let sample = rest.trim().to_string();
        return if sample.is_empty() {
            PreambleLine::Unsupported(line.to_string())
        } else {
            PreambleLine::Commodity { sample }
        };
    }

    if let Some(rest) = trimmed.strip_prefix("account") {
        let name = rest.trim().to_string();
        return if name.is_empty() {
            PreambleLine::Unsupported(line.to_string())
        } else {
            PreambleLine::Account { name }
        };
    }

    if let Some(rest) = trimmed.strip_prefix("include") {
        let path = rest.trim().to_string();
        return if path.is_empty() {
            PreambleLine::Unsupported(line.to_string())
        } else {
            PreambleLine::Include {
                path,
                raw: line.to_string(),
            }
        };
    }

    if let Some(rest) = trimmed.strip_prefix("D ") {
        let value = rest.trim().to_string();
        return if value.is_empty() {
            PreambleLine::Unsupported(line.to_string())
        } else {
            PreambleLine::DefaultCommodity { value }
        };
    }

    PreambleLine::Unsupported(line.to_string())
}

fn parse_supported_lines(
    lines: Vec<String>,
    journals_by_path: &HashMap<String, JournalFileRef>,
) -> ImportedJournal {
    let mut settings = JournalSettingsData::default();
    let mut unsupported = String::new();

    for line in lines {
        match parse_preamble_line(&line) {
            PreambleLine::Unsupported(raw) => unsupported.push_str(&raw),
            PreambleLine::DefaultCommodity { value } => settings.default_commodity = Some(value),
            PreambleLine::DecimalMark { value } => settings.decimal_mark = Some(value),
            PreambleLine::Commodity { sample } => {
                settings.commodities.push(CommoditySetting { sample })
            }
            PreambleLine::Account { name } => settings.accounts.push(AccountSetting { name }),
            PreambleLine::Include { path, raw } => {
                if let Some(file) = journals_by_path.get(&path) {
                    settings.includes.push(file.id);
                } else {
                    unsupported.push_str(&raw);
                }
            }
        }
    }

    ImportedJournal {
        settings,
        unsupported_preamble: unsupported,
        body: String::new(),
        has_managed_header: false,
    }
}

fn split_unmanaged_journal(
    text: &str,
    journals_by_path: &HashMap<String, JournalFileRef>,
) -> ImportedJournal {
    let lines = split_inclusive_lines(text);
    let tx_re = transaction_start_regex();
    let split_idx = lines
        .iter()
        .position(|line| tx_re.is_match(line.trim_start()))
        .unwrap_or(lines.len());

    let mut imported = parse_supported_lines(lines[..split_idx].to_vec(), journals_by_path);
    imported.body = lines[split_idx..].concat();
    imported
}

fn parse_managed_header(
    text: &str,
    journals_by_path: &HashMap<String, JournalFileRef>,
) -> Option<ImportedJournal> {
    let start = text.find(HEADER_BEGIN)?;
    let end_marker_start = text[start..].find(HEADER_END)? + start;
    let end_marker_line_end = text[end_marker_start..]
        .find('\n')
        .map(|idx| end_marker_start + idx + 1)
        .unwrap_or(text.len());

    let managed_start = start + HEADER_BEGIN.len();
    let managed_lines = split_inclusive_lines(&text[managed_start..end_marker_start]);
    let mut imported = parse_supported_lines(managed_lines, journals_by_path);
    imported.has_managed_header = true;
    imported.body = {
        let mut suffix = String::new();
        suffix.push_str(&text[..start]);
        suffix.push_str(&text[end_marker_line_end..]);
        suffix
    };
    Some(imported)
}

fn normalize_settings(input: &JournalSettingsData) -> JournalSettingsData {
    JournalSettingsData {
        default_commodity: input
            .default_commodity
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        decimal_mark: input
            .decimal_mark
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        commodities: input
            .commodities
            .iter()
            .map(|c| CommoditySetting {
                sample: c.sample.trim().to_string(),
            })
            .filter(|c| !c.sample.is_empty())
            .collect(),
        accounts: input
            .accounts
            .iter()
            .map(|a| AccountSetting {
                name: a.name.trim().to_string(),
            })
            .filter(|a| !a.name.is_empty())
            .collect(),
        includes: input.includes.clone(),
    }
}

async fn list_user_journals(
    state: &AppState,
    user_id: i64,
) -> Result<Vec<JournalFileRef>, AppError> {
    let records: Vec<FileRecord> = sqlx::query_as(
        "SELECT id, user_id, filename, file_type, size_bytes, disk_path, created_at \
         FROM files WHERE user_id = ? AND file_type = 'journal'",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    Ok(records
        .into_iter()
        .map(|r| JournalFileRef {
            id: r.id,
            disk_path: r.disk_path,
        })
        .collect())
}

fn journals_by_path(journals: &[JournalFileRef]) -> HashMap<String, JournalFileRef> {
    journals
        .iter()
        .cloned()
        .map(|j| (j.disk_path.clone(), j))
        .collect()
}

async fn fetch_file_record(
    state: &AppState,
    user_id: i64,
    id: i64,
) -> Result<FileRecord, AppError> {
    let record: Option<FileRecord> = sqlx::query_as(
        "SELECT id, user_id, filename, file_type, size_bytes, disk_path, created_at \
         FROM files WHERE id = ? AND user_id = ?",
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?;

    let record = record.ok_or_else(|| AppError::NotFound(format!("file {id}")))?;
    if record.file_type != "journal" {
        return Err(AppError::BadRequest(format!("file {id} is not a journal")));
    }

    Ok(record)
}

async fn fetch_stored_settings(
    state: &AppState,
    file_id: i64,
) -> Result<Option<JournalSettingsData>, AppError> {
    let row: Option<JournalSettingsRow> = sqlx::query_as(
        "SELECT file_id, default_commodity, decimal_mark, commodities_json, accounts_json, includes_json \
         FROM journal_settings WHERE file_id = ?",
    )
    .bind(file_id)
    .fetch_optional(&state.db)
    .await?;

    row.map(map_row_to_data).transpose()
}

async fn effective_settings_for_file(
    state: &AppState,
    file: &FileRecord,
    journals: &[JournalFileRef],
) -> Result<ImportedJournal, AppError> {
    if let Some(settings) = fetch_stored_settings(state, file.id).await? {
        return Ok(ImportedJournal {
            settings,
            unsupported_preamble: String::new(),
            body: tokio::fs::read_to_string(&file.disk_path).await?,
            has_managed_header: tokio::fs::read_to_string(&file.disk_path)
                .await?
                .contains(HEADER_BEGIN),
        });
    }

    let content = tokio::fs::read_to_string(&file.disk_path).await?;
    let by_path = journals_by_path(journals);
    if let Some(imported) = parse_managed_header(&content, &by_path) {
        return Ok(imported);
    }
    Ok(split_unmanaged_journal(&content, &by_path))
}

fn validate_settings_shape(settings: &JournalSettingsData) -> Result<(), AppError> {
    if let Some(mark) = &settings.decimal_mark
        && mark != "."
        && mark != ","
    {
        return Err(AppError::BadRequest(
            "decimal_mark must be '.' or ','".into(),
        ));
    }

    for commodity in &settings.commodities {
        if commodity.sample.trim().is_empty() {
            return Err(AppError::BadRequest(
                "commodity samples cannot be empty".into(),
            ));
        }
    }

    let mut account_names = HashSet::new();
    for account in &settings.accounts {
        let name = account.name.trim();
        if name.is_empty() {
            return Err(AppError::BadRequest("account names cannot be empty".into()));
        }
        if !account_names.insert(name.to_string()) {
            return Err(AppError::BadRequest(format!(
                "duplicate account declaration: {name}"
            )));
        }
    }

    Ok(())
}

async fn validate_include_targets(
    state: &AppState,
    user_id: i64,
    root_file_id: i64,
    settings: &JournalSettingsData,
    journals: &[JournalFileRef],
) -> Result<Vec<String>, AppError> {
    let journal_by_id: HashMap<i64, &JournalFileRef> = journals.iter().map(|j| (j.id, j)).collect();
    let mut rendered = Vec::with_capacity(settings.includes.len());
    let mut seen = HashSet::new();

    for include_id in &settings.includes {
        if *include_id == root_file_id {
            return Err(AppError::BadRequest("journal cannot include itself".into()));
        }
        if !seen.insert(*include_id) {
            return Err(AppError::BadRequest(format!(
                "duplicate include target: {include_id}"
            )));
        }
        let file = journal_by_id.get(include_id).ok_or_else(|| {
            AppError::BadRequest(format!("included journal {include_id} not found"))
        })?;
        rendered.push(file.disk_path.clone());
    }

    let mut stack: Vec<i64> = settings.includes.clone();
    let mut visited = HashSet::new();
    let by_path = journals_by_path(journals);
    while let Some(curr) = stack.pop() {
        if curr == root_file_id {
            return Err(AppError::BadRequest(
                "journal includes create a cycle".into(),
            ));
        }
        if !visited.insert(curr) {
            continue;
        }

        let next = if let Some(stored) = fetch_stored_settings(state, curr).await? {
            stored.includes
        } else {
            let record = fetch_file_record(state, user_id, curr).await?;
            let content = tokio::fs::read_to_string(&record.disk_path).await?;
            if let Some(imported) = parse_managed_header(&content, &by_path) {
                imported.settings.includes
            } else {
                split_unmanaged_journal(&content, &by_path)
                    .settings
                    .includes
            }
        };

        stack.extend(next);
    }

    Ok(rendered)
}

fn render_managed_header(settings: &JournalSettingsData, include_paths: &[String]) -> String {
    let mut out = String::new();
    out.push_str(HEADER_BEGIN);
    out.push('\n');

    if let Some(default_commodity) = &settings.default_commodity {
        out.push_str(&format!(
            "D {}\n",
            render_default_commodity_amount(default_commodity, settings.decimal_mark.as_deref())
        ));
    }
    if let Some(decimal_mark) = &settings.decimal_mark {
        out.push_str(&format!("decimal-mark {decimal_mark}\n"));
    }
    for commodity in &settings.commodities {
        out.push_str(&format!("commodity {}\n", commodity.sample));
    }
    for account in &settings.accounts {
        out.push_str(&format!("account {}\n", account.name));
    }
    for include_path in include_paths {
        out.push_str(&format!("include {include_path}\n"));
    }

    out.push_str(HEADER_END);
    out.push_str("\n\n");
    out
}

fn render_default_commodity_amount(raw: &str, decimal_mark: Option<&str>) -> String {
    let value = raw.trim();
    if value.chars().any(|c| c.is_ascii_digit()) {
        return value.to_string();
    }

    let sample = if decimal_mark == Some(",") {
        "1,00"
    } else {
        "1.00"
    };

    if value
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic())
    {
        format!("{sample} {value}")
    } else {
        format!("{value}{sample}")
    }
}

fn merge_unmanaged_content(header: &str, imported: ImportedJournal) -> String {
    let mut out = String::new();
    out.push_str(header);
    out.push_str(&imported.unsupported_preamble);
    out.push_str(&imported.body);
    out
}

fn replace_managed_header(content: &str, header: &str) -> String {
    let Some(start) = content.find(HEADER_BEGIN) else {
        return format!("{header}{content}");
    };
    let Some(end_start_rel) = content[start..].find(HEADER_END) else {
        return format!("{header}{content}");
    };
    let end_start = start + end_start_rel;
    let end_line_end = content[end_start..]
        .find('\n')
        .map(|idx| end_start + idx + 1)
        .unwrap_or(content.len());

    let mut out = String::new();
    out.push_str(&content[..start]);
    out.push_str(header);
    out.push_str(&content[end_line_end..]);
    out
}

async fn validate_rendered_journal(
    state: &AppState,
    file: &FileRecord,
    content: &str,
) -> Result<(), AppError> {
    let tmp_path = state
        .data_dir
        .join(file.user_id.to_string())
        .join(format!(".tmp_{}_validate.journal", file.id));
    tokio::fs::write(&tmp_path, content).await?;
    let tmp_str = tmp_path.to_string_lossy().into_owned();
    let result = crate::core::hledger::run_raw(&["print", "-f", &tmp_str]).await;
    let _ = tokio::fs::remove_file(&tmp_path).await;
    result.map(|_| ())
}

async fn upsert_settings(
    state: &AppState,
    file_id: i64,
    settings: &JournalSettingsData,
) -> Result<(), AppError> {
    let commodities_json = serde_json::to_string(&settings.commodities)
        .map_err(|e| AppError::Internal(format!("serialize commodities: {e}")))?;
    let accounts_json = serde_json::to_string(&settings.accounts)
        .map_err(|e| AppError::Internal(format!("serialize accounts: {e}")))?;
    let includes_json = serde_json::to_string(&settings.includes)
        .map_err(|e| AppError::Internal(format!("serialize includes: {e}")))?;

    sqlx::query(
        "INSERT INTO journal_settings \
         (file_id, default_commodity, decimal_mark, commodities_json, accounts_json, includes_json) \
         VALUES (?, ?, ?, ?, ?, ?) \
         ON CONFLICT(file_id) DO UPDATE SET \
         default_commodity = excluded.default_commodity, \
         decimal_mark = excluded.decimal_mark, \
         commodities_json = excluded.commodities_json, \
         accounts_json = excluded.accounts_json, \
         includes_json = excluded.includes_json, \
         updated_at = datetime('now')",
    )
    .bind(file_id)
    .bind(settings.default_commodity.as_deref())
    .bind(settings.decimal_mark.as_deref())
    .bind(commodities_json)
    .bind(accounts_json)
    .bind(includes_json)
    .execute(&state.db)
    .await?;

    Ok(())
}

pub async fn write_journal_with_settings(
    state: &AppState,
    file: &FileRecord,
    user_id: i64,
    settings: &JournalSettingsData,
) -> Result<FileInfo, AppError> {
    let normalized = normalize_settings(settings);
    validate_settings_shape(&normalized)?;
    let journals = list_user_journals(state, user_id).await?;
    let include_paths =
        validate_include_targets(state, user_id, file.id, &normalized, &journals).await?;
    let header = render_managed_header(&normalized, &include_paths);

    let existing = tokio::fs::read_to_string(&file.disk_path)
        .await
        .unwrap_or_default();
    let by_path = journals_by_path(&journals);
    let new_content = if existing.contains(HEADER_BEGIN) && existing.contains(HEADER_END) {
        replace_managed_header(&existing, &header)
    } else {
        let imported = split_unmanaged_journal(&existing, &by_path);
        merge_unmanaged_content(&header, imported)
    };

    validate_rendered_journal(state, file, &new_content).await?;
    tokio::fs::write(&file.disk_path, new_content.as_bytes()).await?;
    let size_bytes = new_content.len() as i64;

    sqlx::query("UPDATE files SET size_bytes = ? WHERE id = ?")
        .bind(size_bytes)
        .bind(file.id)
        .execute(&state.db)
        .await?;

    upsert_settings(state, file.id, &normalized).await?;

    Ok(FileInfo {
        id: file.id,
        filename: file.filename.clone(),
        file_type: file.file_type.clone(),
        size_bytes,
        created_at: file.created_at.clone(),
    })
}

pub async fn get_settings(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<JournalSettingsDetail>>, AppError> {
    let file = fetch_file_record(&state, claims.sub, id).await?;

    let settings = if let Some(settings) = fetch_stored_settings(&state, file.id).await? {
        settings
    } else {
        let journals = list_user_journals(&state, claims.sub).await?;
        effective_settings_for_file(&state, &file, &journals)
            .await?
            .settings
    };

    Ok(ApiResponse::success(JournalSettingsDetail {
        file: FileInfo::from(file),
        settings,
    }))
}

pub async fn update_settings(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateJournalSettingsRequest>,
) -> Result<Json<ApiResponse<JournalSettingsDetail>>, AppError> {
    let file = fetch_file_record(&state, claims.sub, id).await?;
    let updated_file =
        write_journal_with_settings(&state, &file, claims.sub, &body.settings).await?;

    Ok(ApiResponse::success(JournalSettingsDetail {
        file: updated_file,
        settings: normalize_settings(&body.settings),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ref_map(entries: &[(&str, i64)]) -> HashMap<String, JournalFileRef> {
        entries
            .iter()
            .map(|(path, id)| {
                (
                    (*path).to_string(),
                    JournalFileRef {
                        id: *id,
                        disk_path: (*path).to_string(),
                    },
                )
            })
            .collect()
    }

    #[test]
    fn render_header_uses_stable_order() {
        let settings = JournalSettingsData {
            default_commodity: Some("USD".into()),
            decimal_mark: Some(",".into()),
            commodities: vec![CommoditySetting {
                sample: "1.000,00 EUR".into(),
            }],
            accounts: vec![AccountSetting {
                name: "assets:cash".into(),
            }],
            includes: vec![2],
        };

        let rendered = render_managed_header(&settings, &["/data/2.journal".into()]);
        assert!(rendered.contains("D 1,00 USD\ndecimal-mark ,\ncommodity 1.000,00 EUR\naccount assets:cash\ninclude /data/2.journal\n"));
    }

    #[test]
    fn imports_supported_directives_from_preamble() {
        let text = "D 1.00 USD\ndecimal-mark ,\ncommodity 1.000,00 EUR\naccount assets:cash\ninclude /a/other.journal\n; keep me\n\n2026-01-01 Opening\n  assets:cash  1 USD\n";
        let imported = split_unmanaged_journal(text, &ref_map(&[("/a/other.journal", 7)]));
        assert_eq!(
            imported.settings.default_commodity.as_deref(),
            Some("1.00 USD")
        );
        assert_eq!(imported.settings.decimal_mark.as_deref(), Some(","));
        assert_eq!(imported.settings.includes, vec![7]);
        assert_eq!(imported.settings.accounts[0].name, "assets:cash");
        assert_eq!(imported.unsupported_preamble, "; keep me\n\n");
        assert!(imported.body.starts_with("2026-01-01 Opening"));
    }

    #[test]
    fn unresolved_include_is_preserved_as_unsupported() {
        let text = "include ./relative.journal\naccount assets:cash\n";
        let imported = split_unmanaged_journal(text, &HashMap::new());
        assert_eq!(imported.settings.includes, Vec::<i64>::new());
        assert_eq!(
            imported.unsupported_preamble,
            "include ./relative.journal\n"
        );
    }

    #[test]
    fn replace_only_managed_block() {
        let content = format!(
            "{HEADER_BEGIN}\nD 1.00 USD\n{HEADER_END}\n\n; tail\n2026-01-01 Test\n  assets:cash  1 USD\n"
        );
        let replaced = replace_managed_header(
            &content,
            &format!("{HEADER_BEGIN}\nD 1.00 EUR\n{HEADER_END}\n\n"),
        );
        assert!(replaced.contains("D 1.00 EUR"));
        assert!(replaced.contains("; tail\n2026-01-01 Test"));
        assert!(!replaced.contains("D 1.00 USD"));
    }

    #[test]
    fn render_default_commodity_symbol_as_valid_d_amount() {
        let settings = JournalSettingsData {
            default_commodity: Some("DKK".into()),
            decimal_mark: Some(",".into()),
            ..Default::default()
        };

        let rendered = render_managed_header(&settings, &[]);
        assert!(rendered.contains("D 1,00 DKK\n"));
    }
}
