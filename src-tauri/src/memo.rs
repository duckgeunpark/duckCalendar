use crate::AppState;
use chrono::Utc;
use rusqlite::{Connection, Row};
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

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

/// Distinct dates ('YYYY-MM-DD') that have at least one memo in the given month.
#[tauri::command]
pub fn list_memo_dates(
    state: State<'_, AppState>,
    year: i32,
    month: u32,
) -> Result<Vec<String>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let prefix = format!("{year:04}-{month:02}-%");
    let mut stmt = conn
        .prepare("SELECT DISTINCT target_date FROM memos WHERE target_date LIKE ?1 ORDER BY target_date")
        .map_err(|e| e.to_string())?;
    let dates = stmt
        .query_map([prefix], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?
        .collect::<rusqlite::Result<Vec<String>>>()
        .map_err(|e| e.to_string())?;
    Ok(dates)
}

#[tauri::command]
pub fn list_memos_by_date(
    state: State<'_, AppState>,
    date: String,
) -> Result<Vec<Memo>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let sql = format!("SELECT {COLUMNS} FROM memos WHERE target_date = ?1 ORDER BY created_at");
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let memos = stmt
        .query_map([date], |row| row_to_memo(row))
        .map_err(|e| e.to_string())?
        .collect::<rusqlite::Result<Vec<Memo>>>()
        .map_err(|e| e.to_string())?;
    Ok(memos)
}

#[derive(Debug, Deserialize)]
pub struct NewMemo {
    pub target_date: String,
    pub title: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub is_calendar_event: bool,
    #[serde(default)]
    pub sync_enabled: bool,
}

#[tauri::command]
pub fn create_memo(state: State<'_, AppState>, input: NewMemo) -> Result<Memo, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();
    let memo = Memo {
        memo_id: Uuid::new_v4().to_string(),
        target_date: input.target_date,
        title: input.title,
        content: input.content,
        created_at: now.clone(),
        updated_at: now,
        is_calendar_event: input.is_calendar_event,
        sync_enabled: input.sync_enabled,
    };
    conn.execute(
        "INSERT INTO memos (memo_id, target_date, title, content, created_at, updated_at, is_calendar_event, sync_enabled)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            memo.memo_id,
            memo.target_date,
            memo.title,
            memo.content,
            memo.created_at,
            memo.updated_at,
            memo.is_calendar_event as i64,
            memo.sync_enabled as i64,
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(memo)
}

#[derive(Debug, Deserialize)]
pub struct MemoUpdate {
    pub memo_id: String,
    pub target_date: String,
    pub title: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub is_calendar_event: bool,
    #[serde(default)]
    pub sync_enabled: bool,
}

#[tauri::command]
pub fn update_memo(state: State<'_, AppState>, input: MemoUpdate) -> Result<Memo, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();
    let affected = conn
        .execute(
            "UPDATE memos
             SET target_date = ?2, title = ?3, content = ?4, updated_at = ?5,
                 is_calendar_event = ?6, sync_enabled = ?7
             WHERE memo_id = ?1",
            rusqlite::params![
                input.memo_id,
                input.target_date,
                input.title,
                input.content,
                now,
                input.is_calendar_event as i64,
                input.sync_enabled as i64,
            ],
        )
        .map_err(|e| e.to_string())?;
    if affected == 0 {
        return Err(format!("memo not found: {}", input.memo_id));
    }
    fetch_memo(&conn, &input.memo_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "memo disappeared after update".to_string())
}

#[tauri::command]
pub fn delete_memo(state: State<'_, AppState>, memo_id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    // google_sync_map row is removed via ON DELETE CASCADE.
    conn.execute("DELETE FROM memos WHERE memo_id = ?1", [&memo_id])
        .map_err(|e| e.to_string())?;
    Ok(())
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
