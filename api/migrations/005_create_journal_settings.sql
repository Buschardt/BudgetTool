CREATE TABLE IF NOT EXISTS journal_settings (
    file_id             INTEGER PRIMARY KEY REFERENCES files(id) ON DELETE CASCADE,
    default_commodity   TEXT,
    decimal_mark        TEXT,
    commodities_json    TEXT NOT NULL DEFAULT '[]',
    accounts_json       TEXT NOT NULL DEFAULT '[]',
    includes_json       TEXT NOT NULL DEFAULT '[]',
    created_at          TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at          TEXT NOT NULL DEFAULT (datetime('now'))
);
