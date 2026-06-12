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

export const WEEKDAYS = ["일", "월", "화", "수", "목", "금", "토"];
export const MONTH_LABELS = [
  "1월", "2월", "3월", "4월", "5월", "6월",
  "7월", "8월", "9월", "10월", "11월", "12월",
];
