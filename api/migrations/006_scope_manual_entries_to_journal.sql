-- Add nullable journal_file_id to the three manual-entry tables
ALTER TABLE commodity_prices ADD COLUMN journal_file_id INTEGER REFERENCES files(id) ON DELETE CASCADE;
ALTER TABLE manual_transactions ADD COLUMN journal_file_id INTEGER REFERENCES files(id) ON DELETE CASCADE;
ALTER TABLE periodic_transactions ADD COLUMN journal_file_id INTEGER REFERENCES files(id) ON DELETE CASCADE;

-- Backfill each row with the user's first (oldest) journal file
UPDATE commodity_prices
SET journal_file_id = (
    SELECT MIN(id) FROM files WHERE user_id = commodity_prices.user_id AND file_type = 'journal'
);
UPDATE manual_transactions
SET journal_file_id = (
    SELECT MIN(id) FROM files WHERE user_id = manual_transactions.user_id AND file_type = 'journal'
);
UPDATE periodic_transactions
SET journal_file_id = (
    SELECT MIN(id) FROM files WHERE user_id = periodic_transactions.user_id AND file_type = 'journal'
);

-- Drop rows whose user had no journal at migration time
DELETE FROM commodity_prices WHERE journal_file_id IS NULL;
DELETE FROM manual_transactions WHERE journal_file_id IS NULL;
DELETE FROM periodic_transactions WHERE journal_file_id IS NULL;

-- Recreate commodity_prices with NOT NULL constraint and a better index
CREATE TABLE commodity_prices_new (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id          INTEGER NOT NULL REFERENCES users(id),
    journal_file_id  INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    date             TEXT    NOT NULL,
    commodity        TEXT    NOT NULL,
    amount           TEXT    NOT NULL,
    target_commodity TEXT    NOT NULL,
    comment          TEXT    NOT NULL DEFAULT '',
    created_at       TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at       TEXT    NOT NULL DEFAULT (datetime('now'))
);
INSERT INTO commodity_prices_new
    SELECT id, user_id, journal_file_id, date, commodity, amount, target_commodity, comment, created_at, updated_at
    FROM commodity_prices;
DROP TABLE commodity_prices;
ALTER TABLE commodity_prices_new RENAME TO commodity_prices;
CREATE INDEX IF NOT EXISTS idx_commodity_prices_journal ON commodity_prices(journal_file_id, date);

-- Recreate manual_transactions with NOT NULL constraint
CREATE TABLE manual_transactions_new (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id         INTEGER NOT NULL REFERENCES users(id),
    journal_file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    date            TEXT    NOT NULL,
    status          TEXT    NOT NULL DEFAULT '',
    code            TEXT    NOT NULL DEFAULT '',
    description     TEXT    NOT NULL,
    comment         TEXT    NOT NULL DEFAULT '',
    postings        TEXT    NOT NULL,
    created_at      TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT    NOT NULL DEFAULT (datetime('now'))
);
INSERT INTO manual_transactions_new
    SELECT id, user_id, journal_file_id, date, status, code, description, comment, postings, created_at, updated_at
    FROM manual_transactions;
DROP TABLE manual_transactions;
ALTER TABLE manual_transactions_new RENAME TO manual_transactions;
CREATE INDEX IF NOT EXISTS idx_manual_transactions_journal ON manual_transactions(journal_file_id, date);

-- Recreate periodic_transactions with NOT NULL constraint
CREATE TABLE periodic_transactions_new (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id         INTEGER NOT NULL REFERENCES users(id),
    journal_file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    period          TEXT    NOT NULL,
    description     TEXT    NOT NULL DEFAULT '',
    comment         TEXT    NOT NULL DEFAULT '',
    postings        TEXT    NOT NULL,
    created_at      TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT    NOT NULL DEFAULT (datetime('now'))
);
INSERT INTO periodic_transactions_new
    SELECT id, user_id, journal_file_id, period, description, comment, postings, created_at, updated_at
    FROM periodic_transactions;
DROP TABLE periodic_transactions;
ALTER TABLE periodic_transactions_new RENAME TO periodic_transactions;
CREATE INDEX IF NOT EXISTS idx_periodic_transactions_journal ON periodic_transactions(journal_file_id);

-- Drop the now-obsolete single-per-user manual entry journal table
DROP TABLE IF EXISTS manual_entry_journals;
