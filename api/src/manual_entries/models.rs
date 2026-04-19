use serde::Serialize;

#[derive(Debug, sqlx::FromRow)]
pub struct CommodityPriceRecord {
    pub id: i64,
    pub user_id: i64,
    pub journal_file_id: i64,
    pub date: String,
    pub commodity: String,
    pub amount: String,
    pub target_commodity: String,
    pub comment: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct CommodityPriceInfo {
    pub id: i64,
    pub journal_file_id: i64,
    pub date: String,
    pub commodity: String,
    pub amount: String,
    pub target_commodity: String,
    pub comment: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<CommodityPriceRecord> for CommodityPriceInfo {
    fn from(r: CommodityPriceRecord) -> Self {
        CommodityPriceInfo {
            id: r.id,
            journal_file_id: r.journal_file_id,
            date: r.date,
            commodity: r.commodity,
            amount: r.amount,
            target_commodity: r.target_commodity,
            comment: r.comment,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct ManualTransactionRecord {
    pub id: i64,
    pub user_id: i64,
    pub journal_file_id: i64,
    pub date: String,
    pub status: String,
    pub code: String,
    pub description: String,
    pub comment: String,
    pub postings: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct ManualTransactionInfo {
    pub id: i64,
    pub journal_file_id: i64,
    pub date: String,
    pub status: String,
    pub code: String,
    pub description: String,
    pub comment: String,
    pub postings: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

impl From<ManualTransactionRecord> for ManualTransactionInfo {
    fn from(r: ManualTransactionRecord) -> Self {
        let postings: serde_json::Value =
            serde_json::from_str(&r.postings).unwrap_or(serde_json::Value::Array(vec![]));
        ManualTransactionInfo {
            id: r.id,
            journal_file_id: r.journal_file_id,
            date: r.date,
            status: r.status,
            code: r.code,
            description: r.description,
            comment: r.comment,
            postings,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct PeriodicTransactionRecord {
    pub id: i64,
    pub user_id: i64,
    pub journal_file_id: i64,
    pub period: String,
    pub description: String,
    pub comment: String,
    pub postings: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct PeriodicTransactionInfo {
    pub id: i64,
    pub journal_file_id: i64,
    pub period: String,
    pub description: String,
    pub comment: String,
    pub postings: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

impl From<PeriodicTransactionRecord> for PeriodicTransactionInfo {
    fn from(r: PeriodicTransactionRecord) -> Self {
        let postings: serde_json::Value =
            serde_json::from_str(&r.postings).unwrap_or(serde_json::Value::Array(vec![]));
        PeriodicTransactionInfo {
            id: r.id,
            journal_file_id: r.journal_file_id,
            period: r.period,
            description: r.description,
            comment: r.comment,
            postings,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}
