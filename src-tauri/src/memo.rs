// Legacy memo model. The app now uses the `event` module; this remains only as
// the data shape for export/sync of the legacy `memos` table until those modules
// are migrated to events (plan Phases 3–4).
use rusqlite::{Connection, Row};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memo {
    pub memo_id: String,
    pub target_date: String,
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
    pub is_calendar_event: bool,
    pub sync_enabled: bool,
}

const COLUMNS: &str =
    "memo_id, target_date, title, content, created_at, updated_at, is_calendar_event, sync_enabled";

fn row_to_memo(row: &Row) -> rusqlite::Result<Memo> {
    Ok(Memo {
        memo_id: row.get(0)?,
        target_date: row.get(1)?,
        title: row.get(2)?,
        content: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
        is_calendar_event: row.get::<_, i64>(6)? != 0,
        sync_enabled: row.get::<_, i64>(7)? != 0,
    })
}

/// Fetch a single memo by id (used by sync). Returns None if missing.
pub fn fetch_memo(conn: &Connection, memo_id: &str) -> rusqlite::Result<Option<Memo>> {
    let sql = format!("SELECT {COLUMNS} FROM memos WHERE memo_id = ?1");
    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query([memo_id])?;
    match rows.next()? {
        Some(row) => Ok(Some(row_to_memo(row)?)),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn mem_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        db::open_for_test(&conn);
        conn
    }

    #[test]
    fn insert_and_fetch_roundtrip() {
        let conn = mem_conn();
        let now = "2026-06-12T00:00:00Z";
        conn.execute(
            "INSERT INTO memos (memo_id, target_date, title, content, created_at, updated_at, is_calendar_event, sync_enabled)
             VALUES ('m1', '2026-06-12', 'hello', 'body', ?1, ?1, 1, 0)",
            [now],
        )
        .unwrap();
        let memo = fetch_memo(&conn, "m1").unwrap().unwrap();
        assert_eq!(memo.title, "hello");
        assert!(memo.is_calendar_event);
        assert!(!memo.sync_enabled);
        assert!(fetch_memo(&conn, "missing").unwrap().is_none());
    }
}
