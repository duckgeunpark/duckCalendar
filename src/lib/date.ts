export function pad2(n: number): string {
  return String(n).padStart(2, "0");
}

/** Format y/m/d (1-based month) as 'YYYY-MM-DD'. */
export function fmt(year: number, month: number, day: number): string {
  return `${year}-${pad2(month)}-${pad2(day)}`;
}

export function todayParts(): { y: number; m: number; d: number } {
  const t = new Date();
  return { y: t.getFullYear(), m: t.getMonth() + 1, d: t.getDate() };
}

/** Number of days in a 1-based (year, month). */
export function daysInMonth(year: number, month: number): number {
  return new Date(year, month, 0).getDate();
}

/** Weekday (0=Sun..6=Sat) of the first day of a 1-based (year, month). */
export function firstWeekday(year: number, month: number): number {
  return new Date(year, month - 1, 1).getDay();
}

/** Add `n` days to a 'YYYY-MM-DD' string, returning 'YYYY-MM-DD'. */
export function addDays(dateStr: string, n: number): string {
  const d = new Date(dateStr + "T00:00:00");
  d.setDate(d.getDate() + n);
  return fmt(d.getFullYear(), d.getMonth() + 1, d.getDate());
}

/** Half-open ['YYYY-MM-01', first day of next month) for a 1-based (year, month). */
export function monthRange(year: number, month: number): { start: string; end: string } {
  const start = fmt(year, month, 1);
  const ny = month === 12 ? year + 1 : year;
  const nm = month === 12 ? 1 : month + 1;
  return { start, end: fmt(ny, nm, 1) };
}

/** The calendar date ('YYYY-MM-DD') an event's start falls on. */
export function eventStartDate(startAt: string): string {
  return startAt.slice(0, 10);
}

/** The 7 dates (Sun..Sat) of the week containing `dateStr`. */
export function weekDates(dateStr: string): string[] {
  const d = new Date(dateStr + "T00:00:00");
  const sunday = addDays(dateStr, -d.getDay());
  return Array.from({ length: 7 }, (_, i) => addDays(sunday, i));
}

/** 'HH:MM' for a timed event's RFC3339 start/end; '' for all-day. */
export function timeLabel(rfc3339: string): string {
  if (rfc3339.length <= 10) return "";
  const d = new Date(rfc3339);
  if (Number.isNaN(d.getTime())) return "";
  return `${pad2(d.getHours())}:${pad2(d.getMinutes())}`;
}

/** Build an RFC3339 timestamp with the local timezone offset for date + 'HH:MM'. */
export function toRfc3339(dateStr: string, hhmm: string): string {
  const [h, m] = hhmm.split(":").map(Number);
  const d = new Date(dateStr + "T00:00:00");
  d.setHours(h || 0, m || 0, 0, 0);
  const off = -d.getTimezoneOffset();
  const sign = off >= 0 ? "+" : "-";
  const oh = pad2(Math.floor(Math.abs(off) / 60));
  const om = pad2(Math.abs(off) % 60);
  return `${fmt(d.getFullYear(), d.getMonth() + 1, d.getDate())}T${pad2(d.getHours())}:${pad2(d.getMinutes())}:00${sign}${oh}:${om}`;
}

export const WEEKDAYS = ["일", "월", "화", "수", "목", "금", "토"];
export const MONTH_LABELS = [
  "1월", "2월", "3월", "4월", "5월", "6월",
  "7월", "8월", "9월", "10월", "11월", "12월",
];
