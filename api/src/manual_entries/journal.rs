use std::path::Path;

use crate::core::AppState;
use crate::core::error::AppError;
use crate::manual_entries::generator::{self, PeriodicEntry, PriceEntry, TransactionEntry};
use crate::manual_entries::models::{
    CommodityPriceRecord, ManualTransactionRecord, PeriodicTransactionRecord,
};

pub fn sidecar_path_for(journal_disk_path: &str) -> String {
    journal_disk_path
        .strip_suffix(".journal")
        .map(|base| format!("{base}.manual.journal"))
        .unwrap_or_else(|| format!("{journal_disk_path}.manual.journal"))
}

pub async fn regenerate_journal_for(
    state: &AppState,
    journal_file_id: i64,
) -> Result<(), AppError> {
    let row: Option<(String,)> =
        sqlx::query_as("SELECT disk_path FROM files WHERE id = ? AND file_type = 'journal'")
            .bind(journal_file_id)
            .fetch_optional(&state.db)
            .await?;

    let (journal_disk_path,) =
        row.ok_or_else(|| AppError::NotFound(format!("journal {journal_file_id}")))?;

    let sidecar = sidecar_path_for(&journal_disk_path);

    let price_records: Vec<CommodityPriceRecord> = sqlx::query_as(
        "SELECT id, user_id, journal_file_id, date, commodity, amount, target_commodity, comment, created_at, updated_at \
         FROM commodity_prices WHERE journal_file_id = ? ORDER BY date, id",
    )
    .bind(journal_file_id)
    .fetch_all(&state.db)
    .await?;

    let txn_records: Vec<ManualTransactionRecord> = sqlx::query_as(
        "SELECT id, user_id, journal_file_id, date, status, code, description, comment, postings, created_at, updated_at \
         FROM manual_transactions WHERE journal_file_id = ? ORDER BY date, id",
    )
    .bind(journal_file_id)
    .fetch_all(&state.db)
    .await?;

    let periodic_records: Vec<PeriodicTransactionRecord> = sqlx::query_as(
        "SELECT id, user_id, journal_file_id, period, description, comment, postings, created_at, updated_at \
         FROM periodic_transactions WHERE journal_file_id = ? ORDER BY id",
    )
    .bind(journal_file_id)
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

    if prices.is_empty() && transactions.is_empty() && periodics.is_empty() {
        if Path::new(&sidecar).exists() {
            tokio::fs::remove_file(&sidecar).await.ok();
        }
        return Ok(());
    }

    let journal_text = generator::generate_journal_text(&prices, &transactions, &periodics)?;
    tokio::fs::write(&sidecar, journal_text.as_bytes()).await?;

    Ok(())
}
