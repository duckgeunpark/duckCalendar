use crate::memo::Memo;
use crate::AppState;
use chrono::NaiveDate;
use icalendar::{Calendar, Component, Event, EventLike};
use rusqlite::Connection;
use std::fs;
use tauri::State;

const COLUMNS: &str =
    "memo_id, target_date, title, content, created_at, updated_at, is_calendar_event, sync_enabled";

/// Collect memos for an export scope.
/// - scope = "date"  -> value is "YYYY-MM-DD"
/// - scope = "month" -> value is "YYYY-MM"
/// - scope = "all"   -> value ignored
fn fetch(conn: &Connection, scope: &str, value: &str) -> Result<Vec<Memo>, String> {
    let (where_clause, param): (&str, Option<String>) = match scope {
        "date" => ("WHERE target_date = ?1", Some(value.to_string())),
        "month" => ("WHERE target_date LIKE ?1", Some(format!("{value}-%"))),
        "all" => ("", None),
        other => return Err(format!("unknown export scope: {other}")),
    };
    let sql = format!("SELECT {COLUMNS} FROM memos {where_clause} ORDER BY target_date, created_at");
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;

    let map = |row: &rusqlite::Row| -> rusqlite::Result<Memo> {
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
    };

    let rows = match param {
        Some(p) => stmt
            .query_map([p], map)
            .map_err(|e| e.to_string())?
            .collect::<rusqlite::Result<Vec<Memo>>>(),
        None => stmt
            .query_map([], map)
            .map_err(|e| e.to_string())?
            .collect::<rusqlite::Result<Vec<Memo>>>(),
    };
    rows.map_err(|e| e.to_string())
}

fn to_json(memos: &[Memo]) -> Result<String, String> {
    serde_json::to_string_pretty(memos).map_err(|e| e.to_string())
}

fn csv_escape(field: &str) -> String {
    if field.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

fn to_csv(memos: &[Memo]) -> String {
    let mut out = String::from(
        "memo_id,target_date,title,content,created_at,updated_at,is_calendar_event,sync_enabled\n",
    );
    for m in memos {
        let row = [
            m.memo_id.as_str(),
            m.target_date.as_str(),
            m.title.as_str(),
            m.content.as_str(),
            m.created_at.as_str(),
            m.updated_at.as_str(),
        ];
        let mut line: Vec<String> = row.iter().map(|f| csv_escape(f)).collect();
        line.push(m.is_calendar_event.to_string());
        line.push(m.sync_enabled.to_string());
        out.push_str(&line.join(","));
        out.push('\n');
    }
    out
}

/// Build a VCALENDAR string. Only memos flagged as calendar events become VEVENTs.
fn to_ics(memos: &[Memo]) -> Result<String, String> {
    let mut cal = Calendar::new();
    for m in memos.iter().filter(|m| m.is_calendar_event) {
        let date = NaiveDate::parse_from_str(&m.target_date, "%Y-%m-%d")
            .map_err(|e| format!("invalid date '{}': {e}", m.target_date))?;
        let event = Event::new()
            .uid(&format!("{}@duckcalendar", m.memo_id))
            .summary(&m.title)
            .description(&m.content)
            .all_day(date)
            .done();
        cal.push(event);
    }
    Ok(cal.to_string())
}

fn build(memos: &[Memo], format: &str) -> Result<String, String> {
    match format {
        "json" => to_json(memos),
        "csv" => Ok(to_csv(memos)),
        "ics" => to_ics(memos),
        other => Err(format!("unknown export format: {other}")),
    }
}

/// Return the serialized export content as a string (useful for preview / clipboard).
#[tauri::command]
pub fn export_data(
    state: State<'_, AppState>,
    scope: String,
    value: String,
    format: String,
) -> Result<String, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let memos = fetch(&conn, &scope, &value)?;
    build(&memos, &format)
}

/// Build the export content and write it to `path`.
#[tauri::command]
pub fn export_to_file(
    state: State<'_, AppState>,
    scope: String,
    value: String,
    format: String,
    path: String,
) -> Result<(), String> {
    let content = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let memos = fetch(&conn, &scope, &value)?;
        build(&memos, &format)?
    };
    fs::write(&path, content).map_err(|e| format!("failed to write {path}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<Memo> {
        vec![
            Memo {
                memo_id: "m1".into(),
                target_date: "2026-06-12".into(),
                title: "회의, 점심".into(), // contains comma -> must be quoted in CSV
                content: "line1\nline2".into(),
                created_at: "2026-06-12T00:00:00Z".into(),
                updated_at: "2026-06-12T00:00:00Z".into(),
                is_calendar_event: true,
                sync_enabled: false,
            },
            Memo {
                memo_id: "m2".into(),
                target_date: "2026-06-13".into(),
                title: "just a note".into(),
                content: "".into(),
                created_at: "2026-06-13T00:00:00Z".into(),
                updated_at: "2026-06-13T00:00:00Z".into(),
                is_calendar_event: false,
                sync_enabled: false,
            },
        ]
    }

    #[test]
    fn json_is_valid() {
        let s = to_json(&sample()).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(parsed.as_array().unwrap().len(), 2);
    }

    #[test]
    fn csv_quotes_special_fields() {
        let s = to_csv(&sample());
        assert!(s.contains("\"회의, 점심\""));
        assert!(s.contains("\"line1\nline2\""));
        // header + 2 rows
        assert_eq!(s.lines().count(), 3 + 1); // +1 because content has embedded newline
    }

    #[test]
    fn ics_only_includes_calendar_events() {
        let s = to_ics(&sample()).unwrap();
        let vevents = s.matches("BEGIN:VEVENT").count();
        assert_eq!(vevents, 1);
        assert!(s.contains("SUMMARY:회의\\, 점심"));
    }
}
