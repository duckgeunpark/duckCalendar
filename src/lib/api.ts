import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import type {
  CalendarEvent,
  Calendar,
  ExportFormat,
  ExportScope,
  GoogleStatus,
  NewEvent,
  SyncSummary,
} from "./types";

// ---- Calendars -------------------------------------------------------------

export function listCalendars(): Promise<Calendar[]> {
  return invoke("list_calendars");
}

export function setCalendarVisible(calendarId: string, visible: boolean): Promise<void> {
  return invoke("set_calendar_visible", { calendarId, visible });
}

// ---- Events ----------------------------------------------------------------

/** Events overlapping the half-open range [start, end) (both 'YYYY-MM-DD' or RFC3339). */
export function listEventsByRange(start: string, end: string): Promise<CalendarEvent[]> {
  return invoke("list_events_by_range", { start, end });
}

export function createEvent(input: NewEvent): Promise<CalendarEvent> {
  return invoke("create_event", { input });
}

export function updateEvent(ev: CalendarEvent): Promise<CalendarEvent> {
  return invoke("update_event", {
    input: {
      event_id: ev.event_id,
      calendar_id: ev.calendar_id,
      title: ev.title,
      description: ev.description,
      location: ev.location,
      start_at: ev.start_at,
      end_at: ev.end_at,
      all_day: ev.all_day,
      color: ev.color,
      sync_enabled: ev.sync_enabled,
    },
  });
}

export function deleteEvent(eventId: string): Promise<void> {
  return invoke("delete_event", { eventId });
}

// ---- Settings --------------------------------------------------------------

export function getSetting(key: string): Promise<string | null> {
  return invoke("get_setting", { key });
}

export function setSetting(key: string, value: string): Promise<void> {
  return invoke("set_setting", { key, value });
}

export type WindowMode = "normal" | "top" | "desktop";

export function setWindowMode(mode: WindowMode): Promise<void> {
  return invoke("set_window_mode", { mode });
}

export function setExpanded(expanded: boolean): Promise<void> {
  return invoke("set_expanded", { expanded });
}

// ---- Export ----------------------------------------------------------------

const EXT: Record<ExportFormat, string> = { json: "json", csv: "csv", ics: "ics" };

/**
 * Prompt for a save path and write the export there.
 * Returns the chosen path, or null if the user cancelled.
 */
export async function exportToFile(
  scope: ExportScope,
  value: string,
  format: ExportFormat,
): Promise<string | null> {
  const suggested = `duckCalendar-${scope}${value ? "-" + value : ""}.${EXT[format]}`;
  const path = await save({
    defaultPath: suggested,
    filters: [{ name: format.toUpperCase(), extensions: [EXT[format]] }],
  });
  if (!path) return null;
  await invoke("export_to_file", { scope, value, format, path });
  return path;
}

// ---- Google Calendar -------------------------------------------------------

export function googleStatus(): Promise<GoogleStatus> {
  return invoke("google_status");
}

export function googleConnect(): Promise<void> {
  return invoke("google_connect");
}

export function setGoogleClientId(clientId: string): Promise<void> {
  return invoke("set_google_client_id", { clientId });
}

export function googleDisconnect(): Promise<void> {
  return invoke("google_disconnect");
}

export function syncMemo(memoId: string): Promise<string> {
  return invoke("sync_memo", { memoId });
}

export function syncSelected(): Promise<SyncSummary> {
  return invoke("sync_selected");
}

export function syncStatusMap(): Promise<Record<string, string>> {
  return invoke("sync_status_map");
}
