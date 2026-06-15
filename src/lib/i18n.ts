import { ui } from "./store.svelte";

// Reading `ui.lang` inside these helpers (called from component markup) keeps
// the UI reactive: switching language re-renders everything that uses them.

const WD: Record<string, string[]> = {
  ko: ["일", "월", "화", "수", "목", "금", "토"],
  en: ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"],
};

const MON_SHORT: Record<string, string[]> = {
  ko: ["1월", "2월", "3월", "4월", "5월", "6월", "7월", "8월", "9월", "10월", "11월", "12월"],
  en: ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"],
};

const MON_FULL_EN = [
  "January", "February", "March", "April", "May", "June",
  "July", "August", "September", "October", "November", "December",
];

type Entry = { ko: string; en: string };

const S: Record<string, Entry> = {
  // App / window
  expandDetailed: { ko: "자세히보기", en: "Detailed" },
  expandCompact: { ko: "간략보기", en: "Compact" },
  settings: { ko: "설정", en: "Settings" },
  hide: { ko: "숨기기", en: "Hide" },
  close: { ko: "닫기", en: "Close" },
  viewMonth: { ko: "월", en: "Month" },
  viewWeek: { ko: "주", en: "Week" },
  viewDay: { ko: "일", en: "Day" },
  goToday: { ko: "오늘로 이동", en: "Go to today" },

  // Calendar
  prevMonth: { ko: "이전 달", en: "Previous month" },
  nextMonth: { ko: "다음 달", en: "Next month" },
  selectMonth: { ko: "월 선택", en: "Select month" },
  prevYear: { ko: "이전 해", en: "Previous year" },
  nextYear: { ko: "다음 해", en: "Next year" },
  clickToViewDay: { ko: "클릭하여 그날 보기", en: "Click to view that day" },

  // DayView
  allDayBadge: { ko: "종일", en: "All day" },
  noEvents: { ko: "일정이 없습니다. 위 버튼으로 추가하세요.", en: "No events. Add one with the button above." },
  monthViewBtn: { ko: "월별 ▦", en: "Month ▦" },
  monthViewTitle: { ko: "월별 보기 (우클릭으로도 가능)", en: "Month view (or right-click)" },
  addEvent: { ko: "＋ 새 일정", en: "＋ New event" },

  // TimeGrid
  addAllDay: { ko: "클릭하여 종일 일정 추가", en: "Click to add an all-day event" },

  // EventEditor
  backList: { ko: "‹ 목록", en: "‹ List" },
  backListTitle: { ko: "목록으로", en: "Back to list" },
  editEvent: { ko: "일정 편집", en: "Edit event" },
  newEvent: { ko: "새 일정", en: "New event" },
  delete: { ko: "삭제", en: "Delete" },
  save: { ko: "저장", en: "Save" },
  cancel: { ko: "취소", en: "Cancel" },
  allDay: { ko: "종일", en: "All day" },
  location: { ko: "위치", en: "Location" },
  titlePh: { ko: "제목", en: "Title" },
  locationPh: { ko: "위치 (선택)", en: "Location (optional)" },
  descPh: { ko: "설명 (선택)", en: "Description (optional)" },
  errEndAfterStart: { ko: "종료 시각이 시작 시각보다 늦어야 합니다.", en: "End time must be after the start time." },
  errTitleRequired: { ko: "제목을 입력하세요.", en: "Please enter a title." },

  // Settings — card group headings
  displayHead: { ko: "화면 설정", en: "Display" },
  readabilityHead: { ko: "가독성 및 효과", en: "Readability & Effects" },
  systemHead: { ko: "시스템 및 데이터 설정", en: "System & Data" },
  languageCardHead: { ko: "언어 설정", en: "Language" },
  styleHead: { ko: "스타일 설정", en: "Style" },
  exportScopeHead: { ko: "데이터 추출 범위", en: "Export range" },
  exportFormatHead: { ko: "데이터 형식", en: "Format" },

  // Settings
  viewModeHead: { ko: "보기 모드", en: "View mode" },
  viewModeHint: {
    ko: "'자세히보기'는 창을 크게 펼쳐 월·주·일 보기와 시간 그리드를 보여주고, '간략보기'는 컴팩트한 위젯으로 돌아갑니다. (상단의 ⊞/⊟ 버튼과 동일)",
    en: "'Detailed' expands the window to show month/week/day views and the time grid; 'Compact' returns to the small widget. (Same as the ⊞/⊟ button.)",
  },
  windowModeHead: { ko: "창 모드", en: "Window mode" },
  wmNormal: { ko: "일반", en: "Normal" },
  wmTop: { ko: "항상 위", en: "Always on top" },
  wmDesktop: { ko: "바탕화면 고정", en: "Pin to desktop" },
  windowModeHint: {
    ko: "'바탕화면 고정'은 다른 창 뒤(바탕화면 레벨)에 머물고, 상단 이름·최소화·닫기 버튼이 숨겨집니다. 닫기(✕)를 누르면 종료되지 않고 트레이로 숨겨지며, 완전 종료는 트레이 메뉴의 '종료'로 합니다.",
    en: "'Pin to desktop' stays behind other windows (at the desktop level) and hides the title and window buttons. Close (✕) hides to the tray instead of quitting; fully quit from the tray menu's 'Quit'.",
  },
  opacity: { ko: "투명도", en: "Opacity" },
  fontSize: { ko: "글자 크기", en: "Font size" },
  themeHead: { ko: "테마", en: "Theme" },
  themeDark: { ko: "다크", en: "Dark" },
  themeLight: { ko: "라이트", en: "Light" },
  themeNavy: { ko: "네이비", en: "Navy" },
  themeSepia: { ko: "세피아", en: "Sepia" },
  accentHead: { ko: "강조색", en: "Accent color" },
  accentDefault: { ko: "테마 기본색", en: "Theme default" },
  accentReset: { ko: "기본색으로", en: "Reset" },
  languageHead: { ko: "언어", en: "Language" },
  langKo: { ko: "한국어", en: "Korean" },
  langEn: { ko: "영어", en: "English" },
  exportHead: { ko: "데이터 추출", en: "Export data" },
  scopeDate: { ko: "이 날짜", en: "This date" },
  scopeMonth: { ko: "이 달", en: "This month" },
  scopeAll: { ko: "전체", en: "All" },
  exportHint: {
    ko: "JSON·CSV는 모든 일정을, ICS는 캘린더 이벤트(VEVENT)로 추출합니다.",
    en: "JSON/CSV export all events; ICS exports calendar events (VEVENT).",
  },
};

/** Translate a static UI string by key. */
export function t(key: keyof typeof S): string {
  const e = S[key];
  return e ? e[ui.lang] : (key as string);
}

/** Short weekday label (0=Sun..6=Sat) in the current language. */
export function wd(i: number): string {
  return WD[ui.lang][i];
}

/** Short month label (0-based month index) in the current language. */
export function monShort(i: number): string {
  return MON_SHORT[ui.lang][i];
}

/** Heading like "2026년 6월" / "June 2026" (1-based month). */
export function monthTitle(year: number, month1: number): string {
  return ui.lang === "en" ? `${MON_FULL_EN[month1 - 1]} ${year}` : `${year}년 ${MON_SHORT.ko[month1 - 1]}`;
}

/** Year heading like "2026년" / "2026". */
export function yearLabel(year: number): string {
  return ui.lang === "en" ? String(year) : `${year}년`;
}
