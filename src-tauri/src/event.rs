use crate::db::LOCAL_CALENDAR_ID;
use crate::AppState;
use chrono::Utc;
use rusqlite::{Connection, Row};
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub event_id: String,
    pub calendar_id: String,
    pub title: String,
    pub description: String,
    pub location: String,
    /// RFC3339 for timed events, 'YYYY-MM-DD' for all-day.
    pub start_at: String,
    /// Exclusive end. All-day end = day after the last day.
    pub end_at: String,
    pub all_day: bool,
    pub color: Option<String>,
    pub source: String,
    pub google_event_id: Option<String>,
    pub google_calendar_id: Option<String>,
    pub etag: Option<String>,
    pub sync_enabled: bool,
    pub sync_status: String,
    pub last_error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Calendar {
    pub calendar_id: String,
    pub name: String,
    pub color: String,
    pub source: String,
    pub google_calendar_id: Option<String>,
    pub visible: bool,
    pub is_primary: bool,
}

const COLUMNS: &str = "event_id, calendar_id, title, description, location, start_at, end_at, all_day, \
     color, source, google_event_id, google_calendar_id, etag, sync_enabled, sync_status, \
     last_error, created_at, updated_at";

fn row_to_event(row: &Row) -> rusqlite::Result<Event> {
    Ok(Event {
        event_id: row.get(0)?,
        calendar_id: row.get(1)?,
        title: row.get(2)?,
        description: row.get(3)?,
        location: row.get(4)?,
        start_at: row.get(5)?,
        end_at: row.get(6)?,
        all_day: row.get::<_, i64>(7)? != 0,
        color: row.get(8)?,
        source: row.get(9)?,
        google_event_id: row.get(10)?,
        google_calendar_id: row.get(11)?,
        etag: row.get(12)?,
        sync_enabled: row.get::<_, i64>(13)? != 0,
        sync_status: row.get(14)?,
        last_error: row.get(15)?,
        created_at: row.get(16)?,
        updated_at: row.get(17)?,
    })
}

/// Fetch one event by id. Returns None if missing.
pub fn fetch_event(conn: &Connection, event_id: &str) -> rusqlite::Result<Option<Event>> {
    let sql = format!("SELECT {COLUMNS} FROM events WHERE event_id = ?1");
    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query([event_id])?;
    match rows.next()? {
        Some(row) => Ok(Some(row_to_event(row)?)),
        None => Ok(None),
    }
}

// ---- Calendars -------------------------------------------------------------

#[tauri::command]
pub fn list_calendars(state: State<'_, AppState>) -> Result<Vec<Calendar>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT calendar_id, name, color, source, google_calendar_id, visible, is_primary \
             FROM calendars ORDER BY is_primary DESC, name",
        )
        .map_err(|e| e.to_string())?;
    let cals = stmt
        .query_map([], |row| {
            Ok(Calendar {
                calendar_id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                source: row.get(3)?,
                google_calendar_id: row.get(4)?,
                visible: row.get::<_, i64>(5)? != 0,
                is_primary: row.get::<_, i64>(6)? != 0,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<rusqlite::Result<Vec<Calendar>>>()
        .map_err(|e| e.to_string())?;
    Ok(cals)
}

#[tauri::command]
pub fn set_calendar_visible(
    state: State<'_, AppState>,
    calendar_id: String,
    visible: bool,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE calendars SET visible = ?2 WHERE calendar_id = ?1",
        rusqlite::params![calendar_id, visible as i64],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

// ---- Events ----------------------------------------------------------------

/// Events overlapping the half-open range [start, end). Bounds are compared
/// lexically, which works for both 'YYYY-MM-DD' and RFC3339 timestamps.
/// Only events on visible calendars are returned.
#[tauri::command]
pub fn list_events_by_range(
    state: State<'_, AppState>,
    start: String,
    end: String,
) -> Result<Vec<Event>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    // Qualify every column with the `e.` alias: `calendar_id` is otherwise
    // ambiguous against the joined `calendars` table.
    let cols = COLUMNS
        .split(", ")
        .map(|c| format!("e.{c}"))
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "SELECT {cols} FROM events e \
         JOIN calendars c ON c.calendar_id = e.calendar_id \
         WHERE c.visible = 1 AND e.start_at < ?2 AND e.end_at > ?1 \
         ORDER BY e.all_day DESC, e.start_at"
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let events = stmt
        .query_map([start, end], row_to_event)
        .map_err(|e| e.to_string())?
        .collect::<rusqlite::Result<Vec<Event>>>()
        .map_err(|e| e.to_string())?;
    Ok(events)
}

#[derive(Debug, Deserialize)]
pub struct NewEvent {
    #[serde(default = "default_calendar")]
    pub calendar_id: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub location: String,
    pub start_at: String,
    pub end_at: String,
    #[serde(default)]
    pub all_day: bool,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub sync_enabled: bool,
}

fn default_calendar() -> String {
    LOCAL_CALENDAR_ID.to_string()
}

#[tauri::command]
pub fn create_event(state: State<'_, AppState>, input: NewEvent) -> Result<Event, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();
    let event = Event {
        event_id: Uuid::new_v4().to_string(),
        calendar_id: input.calendar_id,
        title: input.title,
        description: input.description,
        location: input.location,
        start_at: input.start_at,
        end_at: input.end_at,
        all_day: input.all_day,
        color: input.color,
        source: "local".to_string(),
        google_event_id: None,
        google_calendar_id: None,
        etag: None,
        sync_enabled: input.sync_enabled,
        sync_status: "local".to_string(),
        last_error: None,
        created_at: now.clone(),
        updated_at: now,
    };
    conn.execute(
        "INSERT INTO events (event_id, calendar_id, title, description, location, start_at, end_at, \
            all_day, color, source, sync_enabled, sync_status, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        rusqlite::params![
            event.event_id,
            event.calendar_id,
            event.title,
            event.description,
            event.location,
            event.start_at,
            event.end_at,
            event.all_day as i64,
            event.color,
            event.source,
            event.sync_enabled as i64,
            event.sync_status,
            event.created_at,
            event.updated_at,
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(event)
}

#[derive(Debug, Deserialize)]
pub struct EventUpdate {
    pub event_id: String,
    pub calendar_id: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub location: String,
    pub start_at: String,
    pub end_at: String,
    #[serde(default)]
    pub all_day: bool,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub sync_enabled: bool,
}

#[tauri::command]
pub fn update_event(state: State<'_, AppState>, input: EventUpdate) -> Result<Event, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();
    // Editing a synced event marks it pending so the next sync pushes the change.
    let affected = conn
        .execute(
            "UPDATE events SET \
                calendar_id = ?2, title = ?3, description = ?4, location = ?5, \
                start_at = ?6, end_at = ?7, all_day = ?8, color = ?9, sync_enabled = ?10, \
                updated_at = ?11, \
                sync_status = CASE WHEN sync_status = 'synced' THEN 'pending' ELSE sync_status END \
             WHERE event_id = ?1",
            rusqlite::params![
                input.event_id,
                input.calendar_id,
                input.title,
                input.description,
                input.location,
                input.start_at,
                input.end_at,
                input.all_day as i64,
                input.color,
                input.sync_enabled as i64,
                now,
            ],
        )
        .map_err(|e| e.to_string())?;
    if affected == 0 {
        return Err(format!("event not found: {}", input.event_id));
    }
    fetch_event(&conn, &input.event_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "event disappeared after update".to_string())
}

#[tauri::command]
pub fn delete_event(state: State<'_, AppState>, event_id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM events WHERE event_id = ?1", [&event_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use rusqlite::Connection;

    fn mem_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        db::open_for_test(&conn);
        conn
    }

    fn insert(conn: &Connection, id: &str, start: &str, end: &str, all_day: i64) {
        conn.execute(
            "INSERT INTO events (event_id, calendar_id, title, start_at, end_at, all_day, created_at, updated_at) \
             VALUES (?1, ?2, 't', ?3, ?4, ?5, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            rusqlite::params![id, LOCAL_CALENDAR_ID, start, end, all_day],
        )
        .unwrap();
    }

    #[test]
    fn range_query_includes_overlap_excludes_outside() {
        let conn = mem_conn();
        insert(&conn, "in", "2026-06-12", "2026-06-13", 1); // all-day inside
        insert(&conn, "before", "2026-05-30", "2026-05-31", 1);
        insert(&conn, "timed", "2026-06-15T09:00:00+09:00", "2026-06-15T10:00:00+09:00", 0);

        let sql = "SELECT e.* FROM events e JOIN calendars c ON c.calendar_id = e.calendar_id \
             WHERE c.visible = 1 AND e.start_at < ?2 AND e.end_at > ?1 ORDER BY e.start_at";
        let mut stmt = conn.prepare(sql).unwrap();
        let ids: Vec<String> = stmt
            .query_map(["2026-06-01", "2026-07-01"], |r| r.get::<_, String>(0))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap();
        assert!(ids.contains(&"in".to_string()));
        assert!(ids.contains(&"timed".to_string()));
        assert!(!ids.contains(&"before".to_string()));
    }

    #[test]
    fn hidden_calendar_events_are_excluded() {
        let conn = mem_conn();
        insert(&conn, "e1", "2026-06-12", "2026-06-13", 1);
        conn.execute("UPDATE calendars SET visible = 0 WHERE calendar_id = ?1", [LOCAL_CALENDAR_ID])
            .unwrap();
        let sql = "SELECT e.* FROM events e JOIN calendars c ON c.calendar_id = e.calendar_id \
             WHERE c.visible = 1 AND e.start_at < ?2 AND e.end_at > ?1";
        let mut stmt = conn.prepare(sql).unwrap();
        let n: i64 = stmt
            .query_map(["2026-06-01", "2026-07-01"], |_r| Ok(()))
            .unwrap()
            .count() as i64;
        assert_eq!(n, 0);
    }
}
