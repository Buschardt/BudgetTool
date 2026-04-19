use serde::Serialize;

#[derive(Debug, sqlx::FromRow)]
pub struct FileRecord {
    pub id: i64,
    pub user_id: i64,
    pub filename: String,
    pub file_type: String,
    pub size_bytes: i64,
    pub disk_path: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct FileInfo {
    pub id: i64,
    pub filename: String,
    pub file_type: String,
    pub size_bytes: i64,
    pub created_at: String,
}

impl From<FileRecord> for FileInfo {
    fn from(r: FileRecord) -> Self {
        FileInfo {
            id: r.id,
            filename: r.filename,
            file_type: r.file_type,
            size_bytes: r.size_bytes,
            created_at: r.created_at,
        }
    }
}
