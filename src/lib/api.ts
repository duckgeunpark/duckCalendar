import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import type {
  ExportFormat,
  ExportScope,
  GoogleStatus,
  Memo,
  NewMemo,
  SyncSummary,
} from "./types";

// ---- Memos -----------------------------------------------------------------

export function listMemoDates(year: number, month: number): Promise<string[]> {
  return invoke("list_memo_dates", { year, month });
}

export function listMemosByMonth(year: number, month: number): Promise<Memo[]> {
  return invoke("list_memos_by_month", { year, month });
}

export function listMemosByDate(date: string): Promise<Memo[]> {
  return invoke("list_memos_by_date", { date });
}

export function createMemo(input: NewMemo): Promise<Memo> {
  return invoke("create_memo", { input });
}

export function updateMemo(memo: Memo): Promise<Memo> {
  return invoke("update_memo", {
    input: {
      memo_id: memo.memo_id,
      target_date: memo.target_date,
      title: memo.title,
      content: memo.content,
      is_calendar_event: memo.is_calendar_event,
      sync_enabled: memo.sync_enabled,
    },
  });
}

export function deleteMemo(memoId: string): Promise<void> {
  return invoke("delete_memo", { memoId });
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

export function openMemoWindow(date: string): Promise<void> {
  return invoke("open_memo_window", { date });
}

export function openSettingsWindow(date: string): Promise<void> {
  return invoke("open_settings_window", { date });
}

export function getViewDate(view: string): Promise<string> {
  return invoke("get_view_date", { view });
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
