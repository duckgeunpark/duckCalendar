export interface Calendar {
  calendar_id: string;
  name: string;
  color: string;
  source: "local" | "google";
  google_calendar_id: string | null;
  visible: boolean;
  is_primary: boolean;
}

export interface CalendarEvent {
  event_id: string;
  calendar_id: string;
  title: string;
  description: string;
  location: string;
  /** RFC3339 for timed events, 'YYYY-MM-DD' for all-day. */
  start_at: string;
  /** Exclusive end. All-day end = day after the last day. */
  end_at: string;
  all_day: boolean;
  color: string | null;
  source: "local" | "google";
  google_event_id: string | null;
  google_calendar_id: string | null;
  etag: string | null;
  sync_enabled: boolean;
  sync_status: string;
  last_error: string | null;
  created_at: string;
  updated_at: string;
}

export interface NewEvent {
  calendar_id?: string;
  title: string;
  description: string;
  location: string;
  start_at: string;
  end_at: string;
  all_day: boolean;
  color?: string | null;
  sync_enabled: boolean;
}

export interface GoogleStatus {
  configured: boolean;
  connected: boolean;
}

export interface SyncSummary {
  synced: number;
  failed: number;
  errors: string[];
}

export type ExportScope = "date" | "month" | "all";
export type ExportFormat = "json" | "csv" | "ics";
