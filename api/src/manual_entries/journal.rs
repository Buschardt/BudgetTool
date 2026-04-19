use uuid::Uuid;

use crate::core::AppState;
use crate::core::error::AppError;
use crate::manual_entries::generator::{self, PeriodicEntry, PriceEntry, TransactionEntry};
use crate::manual_entries::models::{
    CommodityPriceRecord, ManualTransactionRecord, PeriodicTransactionRecord,
};

pub async fn regenerate_journal(state: &AppState, user_id: i64) -> Result<(), AppError> {
    let price_records: Vec<CommodityPriceRecord> = sqlx::query_as(
        "SELECT id, user_id, date, commodity, amount, target_commodity, comment, created_at, updated_at \
         FROM commodity_prices WHERE user_id = ? ORDER BY date, id",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    let txn_records: Vec<ManualTransactionRecord> = sqlx::query_as(
        "SELECT id, user_id, date, status, code, description, comment, postings, created_at, updated_at \
         FROM manual_transactions WHERE user_id = ? ORDER BY date, id",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    let periodic_records: Vec<PeriodicTransactionRecord> = sqlx::query_as(
        "SELECT id, user_id, period, description, comment, postings, created_at, updated_at \
         FROM periodic_transactions WHERE user_id = ? ORDER BY id",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

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

    let journal_text = generator::generate_journal_text(&prices, &transactions, &periodics)?;

    let existing: Option<(String,)> =
        sqlx::query_as("SELECT disk_path FROM manual_entry_journals WHERE user_id = ?")
            .bind(user_id)
            .fetch_optional(&state.db)
            .await?;

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
            .await?;
        path_str
    };

    tokio::fs::write(&disk_path, journal_text.as_bytes()).await?;

    sqlx::query("UPDATE manual_entry_journals SET updated_at = datetime('now') WHERE user_id = ?")
        .bind(user_id)
        .execute(&state.db)
        .await?;

    Ok(())
}
