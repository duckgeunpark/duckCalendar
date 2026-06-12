use rusqlite::Connection;
use std::path::Path;

/// Open (or create) the SQLite database at `path` and run migrations.
pub fn open<P: AsRef<Path>>(path: P) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA foreign_keys = ON;",
    )?;
    migrate(&conn)?;
    Ok(conn)
}

/// Run migrations on an existing connection. Used by unit tests in other modules.
#[cfg(test)]
pub fn open_for_test(conn: &Connection) {
    migrate(conn).unwrap();
}

/// Create tables if they do not exist. Idempotent.
fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS memos (
            memo_id           TEXT PRIMARY KEY,
            target_date       TEXT NOT NULL,
            title             TEXT NOT NULL,
            content           TEXT NOT NULL DEFAULT '',
            created_at        TEXT NOT NULL,
            updated_at        TEXT NOT NULL,
            is_calendar_event INTEGER NOT NULL DEFAULT 0,
            sync_enabled      INTEGER NOT NULL DEFAULT 0
        );

        CREATE INDEX IF NOT EXISTS idx_memos_date ON memos(target_date);

        CREATE TABLE IF NOT EXISTS google_sync_map (
            memo_id            TEXT PRIMARY KEY
                               REFERENCES memos(memo_id) ON DELETE CASCADE,
            google_event_id    TEXT,
            sync_status        TEXT NOT NULL DEFAULT 'pending',
            last_synced_at     TEXT,
            last_error_message TEXT
        );

        CREATE TABLE IF NOT EXISTS app_settings (
            setting_key   TEXT PRIMARY KEY,
            setting_value TEXT NOT NULL
        );",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrate_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        // Running again must not error.
        migrate(&conn).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('memos','google_sync_map','app_settings')",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 3);
    }
}
