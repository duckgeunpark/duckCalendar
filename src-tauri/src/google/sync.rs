use super::oauth::valid_access_token;
use super::EVENTS_ENDPOINT;
use crate::memo::{self, Memo};
use crate::AppState;
use chrono::{NaiveDate, Utc};
use rusqlite::Connection;
use serde::Serialize;
use std::collections::HashMap;
use std::time::Duration;
use tauri::State;

#[derive(Debug, Serialize)]
pub struct SyncSummary {
    pub synced: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}

fn existing_event_id(conn: &Connection, memo_id: &str) -> rusqlite::Result<Option<String>> {
    let mut stmt =
        conn.prepare("SELECT google_event_id FROM google_sync_map WHERE memo_id = ?1")?;
    let mut rows = stmt.query([memo_id])?;
    match rows.next()? {
        Some(row) => Ok(row.get::<_, Option<String>>(0)?),
        None => Ok(None),
    }
}

fn mark_synced(conn: &Connection, memo_id: &str, event_id: &str) -> rusqlite::Result<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO google_sync_map (memo_id, google_event_id, sync_status, last_synced_at, last_error_message)
         VALUES (?1, ?2, 'synced', ?3, NULL)
         ON CONFLICT(memo_id) DO UPDATE SET
            google_event_id = excluded.google_event_id,
            sync_status = 'synced',
            last_synced_at = excluded.last_synced_at,
            last_error_message = NULL",
        rusqlite::params![memo_id, event_id, now],
    )?;
    Ok(())
}

fn mark_failed(conn: &Connection, memo_id: &str, message: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO google_sync_map (memo_id, sync_status, last_error_message)
         VALUES (?1, 'failed', ?2)
         ON CONFLICT(memo_id) DO UPDATE SET
            sync_status = 'failed',
            last_error_message = excluded.last_error_message",
        rusqlite::params![memo_id, message],
    )?;
    Ok(())
}

/// Push a memo to Google Calendar as an all-day event. Blocking (network).
/// Returns the Google event id on success.
fn push_event(memo: &Memo, existing_event_id: Option<&str>) -> Result<String, String> {
    let token = valid_access_token()?;

    let start = NaiveDate::parse_from_str(&memo.target_date, "%Y-%m-%d")
        .map_err(|e| format!("잘못된 날짜 '{}': {e}", memo.target_date))?;
    let end = start
        .succ_opt()
        .ok_or_else(|| "날짜 범위를 계산할 수 없습니다.".to_string())?;

    let body = serde_json::json!({
        "summary": memo.title,
        "description": memo.content,
        "start": { "date": start.format("%Y-%m-%d").to_string() },
        "end": { "date": end.format("%Y-%m-%d").to_string() },
    });

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;

    let request = match existing_event_id {
        Some(id) => client.put(format!("{EVENTS_ENDPOINT}/{id}")),
        None => client.post(EVENTS_ENDPOINT),
    };

    let resp = request
        .bearer_auth(&token)
        .json(&body)
        .send()
        .map_err(|e| format!("network: Google Calendar 요청 실패: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        let detail = resp.text().unwrap_or_default();
        let kind = match status.as_u16() {
            401 => "auth: 인증이 만료되었습니다. 다시 연결하세요",
            403 => "permission: 권한이 거부되었습니다",
            _ => "Google Calendar 오류",
        };
        return Err(format!("{kind} ({status}): {detail}"));
    }

    let parsed: serde_json::Value = resp.json().map_err(|e| e.to_string())?;
    parsed
        .get("id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "응답에 event id가 없습니다.".to_string())
}

/// Sync one memo and record the result. Returns the Google event id on success.
async fn sync_one(state: &State<'_, AppState>, memo_id: &str) -> Result<String, String> {
    // 1. Read memo + existing event id, then release the lock before network I/O.
    let (memo, existing) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let memo = memo::fetch_memo(&conn, memo_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("memo not found: {memo_id}"))?;
        let existing = existing_event_id(&conn, memo_id).map_err(|e| e.to_string())?;
        (memo, existing)
    };

    if !memo.is_calendar_event {
        return Err("일정으로 지정된 메모만 동기화할 수 있습니다.".to_string());
    }

    // 2. Network call on a blocking thread.
    let memo_for_push = memo.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        push_event(&memo_for_push, existing.as_deref())
    })
    .await
    .map_err(|e| e.to_string())?;

    // 3. Persist the outcome. Failures stay retriable (status = 'failed').
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    match &result {
        Ok(event_id) => mark_synced(&conn, memo_id, event_id).map_err(|e| e.to_string())?,
        Err(msg) => mark_failed(&conn, memo_id, msg).map_err(|e| e.to_string())?,
    }
    result
}

#[tauri::command]
pub async fn sync_memo(state: State<'_, AppState>, memo_id: String) -> Result<String, String> {
    sync_one(&state, &memo_id).await
}

/// Sync every memo flagged with sync_enabled = 1.
#[tauri::command]
pub async fn sync_selected(state: State<'_, AppState>) -> Result<SyncSummary, String> {
    let ids: Vec<String> = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT memo_id FROM memos WHERE sync_enabled = 1 AND is_calendar_event = 1")
            .map_err(|e| e.to_string())?;
        stmt.query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| e.to_string())?
            .collect::<rusqlite::Result<Vec<String>>>()
            .map_err(|e| e.to_string())?
    };

    let mut summary = SyncSummary {
        synced: 0,
        failed: 0,
        errors: Vec::new(),
    };
    for id in ids {
        match sync_one(&state, &id).await {
            Ok(_) => summary.synced += 1,
            Err(e) => {
                summary.failed += 1;
                summary.errors.push(format!("{id}: {e}"));
            }
        }
    }
    Ok(summary)
}

/// Map of memo_id -> sync_status, for UI badges.
#[tauri::command]
pub fn sync_status_map(state: State<'_, AppState>) -> Result<HashMap<String, String>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT memo_id, sync_status FROM google_sync_map")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?;
    let mut map = HashMap::new();
    for r in rows {
        let (k, v) = r.map_err(|e| e.to_string())?;
        map.insert(k, v);
    }
    Ok(map)
}
