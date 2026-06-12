export interface Memo {
  memo_id: string;
  target_date: string; // 'YYYY-MM-DD'
  title: string;
  content: string;
  created_at: string;
  updated_at: string;
  is_calendar_event: boolean;
  sync_enabled: boolean;
}

export interface NewMemo {
  target_date: string;
  title: string;
  content: string;
  is_calendar_event: boolean;
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
