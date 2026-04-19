CREATE TABLE IF NOT EXISTS manual_entry_journals (
    user_id    INTEGER PRIMARY KEY REFERENCES users(id),
    disk_path  TEXT    NOT NULL,
    updated_at TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS commodity_prices (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id          INTEGER NOT NULL REFERENCES users(id),
    date             TEXT    NOT NULL,
    commodity        TEXT    NOT NULL,
    amount           TEXT    NOT NULL,
    target_commodity TEXT    NOT NULL,
    comment          TEXT    NOT NULL DEFAULT '',
    created_at       TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at       TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_commodity_prices_user ON commodity_prices(user_id);

CREATE TABLE IF NOT EXISTS manual_transactions (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id     INTEGER NOT NULL REFERENCES users(id),
    date        TEXT    NOT NULL,
    status      TEXT    NOT NULL DEFAULT '',
    code        TEXT    NOT NULL DEFAULT '',
    description TEXT    NOT NULL,
    comment     TEXT    NOT NULL DEFAULT '',
    postings    TEXT    NOT NULL,
    created_at  TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_manual_transactions_user ON manual_transactions(user_id);

CREATE TABLE IF NOT EXISTS periodic_transactions (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id     INTEGER NOT NULL REFERENCES users(id),
    period      TEXT    NOT NULL,
    description TEXT    NOT NULL DEFAULT '',
    comment     TEXT    NOT NULL DEFAULT '',
    postings    TEXT    NOT NULL,
    created_at  TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_periodic_transactions_user ON periodic_transactions(user_id);
