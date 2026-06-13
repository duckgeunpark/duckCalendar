use rusqlite::Connection;
use std::path::Path;

/// Identifier of the always-present local calendar.
pub const LOCAL_CALENDAR_ID: &str = "local-default";

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
    conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
    migrate(conn).unwrap();
}

/// Create tables if they do not exist and migrate legacy data. Idempotent.
fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        // ---- Legacy tables (kept so old data can be migrated and as a backup) ----
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
        CREATE TABLE IF NOT EXISTS google_sync_map (
            memo_id            TEXT PRIMARY KEY
                               REFERENCES memos(memo_id) ON DELETE CASCADE,
            google_event_id    TEXT,
            sync_status        TEXT NOT NULL DEFAULT 'pending',
            last_synced_at     TEXT,
            last_error_message TEXT
        );

        -- ---- Calendar/event model (GCal-style) ----
        CREATE TABLE IF NOT EXISTS calendars (
            calendar_id        TEXT PRIMARY KEY,
            name               TEXT NOT NULL,
            color              TEXT NOT NULL DEFAULT '#4285F4',
            source             TEXT NOT NULL DEFAULT 'local',  -- 'local' | 'google'
            google_calendar_id TEXT,
            visible            INTEGER NOT NULL DEFAULT 1,
            is_primary         INTEGER NOT NULL DEFAULT 0,
            sync_token         TEXT
        );

        CREATE TABLE IF NOT EXISTS events (
            event_id           TEXT PRIMARY KEY,
            calendar_id        TEXT NOT NULL
                               REFERENCES calendars(calendar_id) ON DELETE CASCADE,
            title              TEXT NOT NULL,
            description        TEXT NOT NULL DEFAULT '',
            location           TEXT NOT NULL DEFAULT '',
            -- start_at/end_at: RFC3339 for timed events, 'YYYY-MM-DD' for all-day.
            -- end_at is exclusive (all-day end = day after the last day).
            start_at           TEXT NOT NULL,
            end_at             TEXT NOT NULL,
            all_day            INTEGER NOT NULL DEFAULT 0,
            color              TEXT,
            source             TEXT NOT NULL DEFAULT 'local',  -- 'local' | 'google'
            google_event_id    TEXT,
            google_calendar_id TEXT,
            etag               TEXT,
            sync_enabled       INTEGER NOT NULL DEFAULT 0,
            sync_status        TEXT NOT NULL DEFAULT 'local',  -- local|pending|synced|failed
            last_error         TEXT,
            created_at         TEXT NOT NULL,
            updated_at         TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_events_start ON events(start_at);
        CREATE INDEX IF NOT EXISTS idx_events_calendar ON events(calendar_id);

        CREATE TABLE IF NOT EXISTS app_settings (
            setting_key   TEXT PRIMARY KEY,
            setting_value TEXT NOT NULL
        );",
    )?;

    seed_local_calendar(conn)?;
    migrate_memos_to_events(conn)?;
    Ok(())
}

/// Ensure the default local calendar row exists.
fn seed_local_calendar(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO calendars
            (calendar_id, name, color, source, visible, is_primary)
         VALUES (?1, '내 캘린더', '#4285F4', 'local', 1, 1)",
        [LOCAL_CALENDAR_ID],
    )?;
    Ok(())
}

/// One-time copy of legacy `memos` into `events` as all-day events.
/// Safe to run repeatedly: only runs when `events` is empty and `memos` has rows,
/// so it never clobbers data created under the new model.
fn migrate_memos_to_events(conn: &Connection) -> rusqlite::Result<()> {
    let events_count: i64 = conn.query_row("SELECT COUNT(*) FROM events", [], |r| r.get(0))?;
    let memos_count: i64 = conn.query_row("SELECT COUNT(*) FROM memos", [], |r| r.get(0))?;
    if events_count > 0 || memos_count == 0 {
        return Ok(());
    }
    // end_at = day after target_date (exclusive, all-day convention).
    conn.execute(
        "INSERT INTO events (
            event_id, calendar_id, title, description, location,
            start_at, end_at, all_day, source,
            google_event_id, sync_enabled, sync_status, created_at, updated_at
         )
         SELECT
            m.memo_id, ?1, m.title, m.content, '',
            m.target_date, date(m.target_date, '+1 day'), 1, 'local',
            g.google_event_id,
            m.sync_enabled,
            CASE WHEN g.sync_status IS NOT NULL THEN g.sync_status ELSE 'local' END,
            m.created_at, m.updated_at
         FROM memos m
         LEFT JOIN google_sync_map g ON g.memo_id = m.memo_id",
        [LOCAL_CALENDAR_ID],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mem() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        open_for_test(&conn);
        conn
    }

    #[test]
    fn migrate_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        migrate(&conn).unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('memos','google_sync_map','app_settings','calendars','events')",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 5);
    }

    #[test]
    fn seeds_one_local_calendar() {
        let conn = mem();
        let n: i64 = conn
            .query_row("SELECT COUNT(*) FROM calendars WHERE calendar_id = ?1", [LOCAL_CALENDAR_ID], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 1);
    }

    #[test]
    fn migrates_legacy_memos_as_all_day_events() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        // Create only legacy tables, insert a memo, then run full migrate.
        conn.execute_batch(
            "CREATE TABLE memos (memo_id TEXT PRIMARY KEY, target_date TEXT NOT NULL, title TEXT NOT NULL,
                content TEXT NOT NULL DEFAULT '', created_at TEXT NOT NULL, updated_at TEXT NOT NULL,
                is_calendar_event INTEGER NOT NULL DEFAULT 0, sync_enabled INTEGER NOT NULL DEFAULT 0);",
        )
        .unwrap();
        conn.execute(
            "INSERT INTO memos (memo_id, target_date, title, content, created_at, updated_at, is_calendar_event, sync_enabled)
             VALUES ('m1','2026-06-12','old memo','body','2026-06-12T00:00:00Z','2026-06-12T00:00:00Z',1,1)",
            [],
        )
        .unwrap();
        migrate(&conn).unwrap();

        let (start, end, all_day, title): (String, String, i64, String) = conn
            .query_row(
                "SELECT start_at, end_at, all_day, title FROM events WHERE event_id='m1'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .unwrap();
        assert_eq!(start, "2026-06-12");
        assert_eq!(end, "2026-06-13");
        assert_eq!(all_day, 1);
        assert_eq!(title, "old memo");

        // Idempotent: running again does not duplicate.
        migrate(&conn).unwrap();
        let n: i64 = conn.query_row("SELECT COUNT(*) FROM events", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 1);
    }
}
