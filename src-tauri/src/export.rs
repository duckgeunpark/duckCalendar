use crate::AppState;
use chrono::{DateTime, NaiveDate, Utc};
use icalendar::{Calendar, Component, Event, EventLike};
use rusqlite::Connection;
use serde::Serialize;
use std::fs;
use tauri::State;

/// A flattened event row for export (subset of the full event model).
#[derive(Debug, Serialize)]
struct ExportEvent {
    event_id: String,
    calendar_id: String,
    title: String,
    description: String,
    location: String,
    start_at: String,
    end_at: String,
    all_day: bool,
    source: String,
}

const COLUMNS: &str =
    "event_id, calendar_id, title, description, location, start_at, end_at, all_day, source";

/// Collect events for an export scope, matched by the event's start date.
/// - scope = "date"  -> value is "YYYY-MM-DD"
/// - scope = "month" -> value is "YYYY-MM"
/// - scope = "all"   -> value ignored
fn fetch(conn: &Connection, scope: &str, value: &str) -> Result<Vec<ExportEvent>, String> {
    let (where_clause, param): (&str, Option<String>) = match scope {
        "date" => ("WHERE start_at LIKE ?1", Some(format!("{value}%"))),
        "month" => ("WHERE start_at LIKE ?1", Some(format!("{value}%"))),
        "all" => ("", None),
        other => return Err(format!("unknown export scope: {other}")),
    };
    let sql = format!("SELECT {COLUMNS} FROM events {where_clause} ORDER BY start_at");
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;

    let map = |row: &rusqlite::Row| -> rusqlite::Result<ExportEvent> {
        Ok(ExportEvent {
            event_id: row.get(0)?,
            calendar_id: row.get(1)?,
            title: row.get(2)?,
            description: row.get(3)?,
            location: row.get(4)?,
            start_at: row.get(5)?,
            end_at: row.get(6)?,
            all_day: row.get::<_, i64>(7)? != 0,
            source: row.get(8)?,
        })
    };

    let rows = match param {
        Some(p) => stmt
            .query_map([p], map)
            .map_err(|e| e.to_string())?
            .collect::<rusqlite::Result<Vec<ExportEvent>>>(),
        None => stmt
            .query_map([], map)
            .map_err(|e| e.to_string())?
            .collect::<rusqlite::Result<Vec<ExportEvent>>>(),
    };
    rows.map_err(|e| e.to_string())
}

fn to_json(events: &[ExportEvent]) -> Result<String, String> {
    serde_json::to_string_pretty(events).map_err(|e| e.to_string())
}

fn csv_escape(field: &str) -> String {
    if field.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

fn to_csv(events: &[ExportEvent]) -> String {
    let mut out = String::from(
        "event_id,calendar_id,title,description,location,start_at,end_at,all_day,source\n",
    );
    for e in events {
        let mut line: Vec<String> = [
            e.event_id.as_str(),
            e.calendar_id.as_str(),
            e.title.as_str(),
            e.description.as_str(),
            e.location.as_str(),
            e.start_at.as_str(),
            e.end_at.as_str(),
        ]
        .iter()
        .map(|f| csv_escape(f))
        .collect();
        line.push(e.all_day.to_string());
        line.push(csv_escape(&e.source));
        out.push_str(&line.join(","));
        out.push('\n');
    }
    out
}

/// Build a VCALENDAR string. All-day events use DATE values; timed events use
/// DTSTART/DTEND timestamps.
fn to_ics(events: &[ExportEvent]) -> Result<String, String> {
    let mut cal = Calendar::new();
    for e in events {
        let mut ev = Event::new();
        ev.uid(&format!("{}@duckcalendar", e.event_id))
            .summary(&e.title)
            .description(&e.description);
        if !e.location.is_empty() {
            ev.location(&e.location);
        }
        if e.all_day {
            let d = NaiveDate::parse_from_str(&e.start_at, "%Y-%m-%d")
                .map_err(|err| format!("invalid date '{}': {err}", e.start_at))?;
            ev.all_day(d);
        } else {
            let s = DateTime::parse_from_rfc3339(&e.start_at)
                .map_err(|err| format!("invalid start '{}': {err}", e.start_at))?
                .with_timezone(&Utc);
            let en = DateTime::parse_from_rfc3339(&e.end_at)
                .map_err(|err| format!("invalid end '{}': {err}", e.end_at))?
                .with_timezone(&Utc);
            ev.starts(s).ends(en);
        }
        cal.push(ev.done());
    }
    Ok(cal.to_string())
}

fn build(events: &[ExportEvent], format: &str) -> Result<String, String> {
    match format {
        "json" => to_json(events),
        "csv" => Ok(to_csv(events)),
        "ics" => to_ics(events),
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
    let events = fetch(&conn, &scope, &value)?;
    build(&events, &format)
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
        let events = fetch(&conn, &scope, &value)?;
        build(&events, &format)?
    };
    fs::write(&path, content).map_err(|e| format!("failed to write {path}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<ExportEvent> {
        vec![
            ExportEvent {
                event_id: "e1".into(),
                calendar_id: "local-default".into(),
                title: "회의, 점심".into(), // contains comma -> must be quoted in CSV
                description: "line1\nline2".into(),
                location: "서울".into(),
                start_at: "2026-06-12".into(),
                end_at: "2026-06-13".into(),
                all_day: true,
                source: "local".into(),
            },
            ExportEvent {
                event_id: "e2".into(),
                calendar_id: "local-default".into(),
                title: "standup".into(),
                description: "".into(),
                location: "".into(),
                start_at: "2026-06-13T09:00:00+09:00".into(),
                end_at: "2026-06-13T09:30:00+09:00".into(),
                all_day: false,
                source: "local".into(),
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
    }

    #[test]
    fn ics_includes_all_events_with_times_and_dates() {
        let s = to_ics(&sample()).unwrap();
        let vevents = s.matches("BEGIN:VEVENT").count();
        assert_eq!(vevents, 2);
        // All-day event uses a DATE value; timed event uses a DateTime.
        assert!(s.contains("SUMMARY:회의\\, 점심"));
        assert!(s.contains("20260612"));
        assert!(s.contains("20260613T"));
    }
}
