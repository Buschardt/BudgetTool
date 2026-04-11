CREATE TABLE IF NOT EXISTS files (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id     INTEGER NOT NULL REFERENCES users(id),
    filename    TEXT    NOT NULL,
    file_type   TEXT    NOT NULL CHECK (file_type IN ('journal', 'csv', 'rules')),
    size_bytes  INTEGER NOT NULL,
    disk_path   TEXT    NOT NULL,
    created_at  TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_files_user_id ON files(user_id);
