# duckCalendar → 구글 캘린더형 데스크톱 위젯 진화 플랜

## Context (왜 이 작업을 하는가)

현재 duckCalendar는 **로컬 SQLite의 "날짜별 종일 메모"** 가 단일 기준이고, 구글
연동은 메모를 종일 이벤트로 올리는 **단방향 푸시**만 지원한다. 또한 메모/설정이
별도 Tauri 자식 창으로 열리는데, dev에서 이 자식 창들이 **흰 화면**으로 뜨는
버그가 있다(메모 창은 방금 인라인 방식으로 우회 완료, 설정 창은 아직 동일 버그).

사용자 목표: duckCalendar를 **"구글 캘린더처럼 보이고 동작하는 바탕화면 위젯"**
으로 진화. 확정된 방향:
- **로컬 우선 유지** + UI/UX는 구글 캘린더 스타일이 기본
- 구글 연동은 **선택**이되, 연동 시 **양방향 동기화** + **파일 추출** 가능
- 구글(및 다른) 캘린더 이벤트를 위젯에 **함께 표시(오버레이)**
- 이벤트 모델을 **GCal식 시간 이벤트**(시작/종료 시각, 다중 캘린더, 색상)로 확장
- **위젯 = 요약**, **확장 = 풀 GCal 월/주/일 뷰**

## 목표 아키텍처 요약

- 데이터: 로컬 SQLite가 로컬 이벤트의 source of truth. 구글 이벤트는 로컬에
  캐시해 표시. 양방향 동기화는 선택 기능.
- 단일 창 원칙: 자식 창(흰 화면 버그)을 피하기 위해 **모든 화면을 메인 webview
  안에서** 전환. "확장"은 새 창이 아니라 **메인 창을 크게 리사이즈 + 풀 뷰 모드**.
  (설정도 인라인화 → 현재 깨진 설정 창 문제도 함께 해결)

---

## Phase 0 — 단일 창 기반 정리 (UI 골격)

흰 화면 자식 창 의존을 끊고 이후 단계의 토대를 만든다.

- `src/App.svelte`: 뷰 상태를 `view: "month" | "week" | "day" | "settings"` +
  `expanded: boolean` 로 확장. `expanded`면 메인 창을 크게 리사이즈.
- `src/lib/Settings.svelte`: 자식 창(`SettingsWindow`/`openSettingsWindow`) 대신
  메인 창 인라인 뷰로 전환(방금 `DayView`+`MemoEditor` 인라인화한 패턴 그대로).
- 창 리사이즈/복원은 기존 `set_window_mode`/`restore_window`(`src-tauri/src/lib.rs`)
  패턴 재사용. 확장/축소용 명령 1개 추가(또는 프런트에서 `@tauri-apps/api/window`
  `setSize` 직접 호출).
- 정리: `SettingsWindow.svelte`, `open_settings_window`, `get_view_date`,
  `open_child_window`, capabilities의 `settings`/`memo` 윈도우 항목 제거.

검증: 설정/일별/월별이 모두 메인 창 안에서 흰 화면 없이 전환되는지.

## Phase 1 — 이벤트 데이터 모델로 확장 (로컬)

`memo`(종일) → GCal식 `event`(시간) 로 확장. **하위호환 마이그레이션 포함.**

- `src-tauri/src/db.rs`: 신규 테이블 추가(마이그레이션 idempotent 유지)
  - `events(event_id PK, calendar_id, title, description, start_at, end_at,
    all_day, color, location, source('local'|'google'), google_event_id,
    google_calendar_id, etag, updated_at, created_at, sync_enabled, sync_status,
    last_error)`
  - `calendars(calendar_id PK, name, color, source, google_calendar_id,
    visible, is_primary)` — 로컬 기본 캘린더 1개 시드
  - 기존 `memos` → `events` 1회 복사 마이그레이션(종일: `start_at=target_date`,
    `all_day=1`, `source='local'`). `google_sync_map`은 `events`의 컬럼으로 흡수.
- `src-tauri/src/memo.rs` → `event.rs`로 재작성(CRUD + 범위 조회
  `list_events_by_range(start,end)`). `lib.rs`의 `invoke_handler` 갱신.
- 프런트: `src/lib/types.ts`(Event/Calendar 타입), `src/lib/api.ts`(이벤트/캘린더
  CRUD invoke), `src/lib/date.ts`(시간 파싱/포맷 헬퍼 추가).

검증: 기존 메모가 종일 이벤트로 보존되는지, 시간 이벤트 생성/수정/삭제.

## Phase 2 — GCal 스타일 UI (로컬 데이터 기준)

- `src/lib/Calendar.svelte`: 월간 그리드를 GCal풍으로(이벤트 칩 색상 = 캘린더
  색, 종일/시간 구분). 기존 `cells`/`monthMemos` 로직 재사용 후 이벤트 모델로 교체.
- 신규 `WeekView.svelte` / `DayView.svelte`(현 DayView 확장): 시간대 그리드
  (타임라인) + 이벤트 블록. 클릭/드래그로 시간대 선택 → 이벤트 생성.
- `src/lib/MemoEditor.svelte` → `EventEditor.svelte`로 확장: 시작/종료 시각,
  종일 토글, 캘린더 선택, 색상. 기존 저장/삭제/동기화 로직 재사용.
- 위젯 요약 모드: 작은 창에서는 월간 + 오늘 일정 요약. "확장" 버튼 → Phase 0의
  expanded 모드로 풀 월/주/일 뷰.

검증: 월/주/일 전환, 시간 이벤트 표시/생성, 위젯↔확장 토글.

## Phase 3 — 양방향 구글 동기화 + 오버레이

- **스코프 상향**: `src-tauri/src/google/mod.rs`의 `SCOPE`를
  `https://www.googleapis.com/auth/calendar`(읽기/쓰기 + 캘린더 목록)로 변경.
  → 기존 연결 사용자는 **재동의 필요**(연결 해제 후 재연결 안내).
- 풀(가져오기): `google/sync.rs`에 `list_calendars`(CalendarList) +
  `list_events`(per calendar, time range, `syncToken` 증분) 추가 → 로컬 `events`/
  `calendars`에 upsert. `oauth.rs::valid_access_token`(자동 refresh) 재사용.
- 푸시: 기존 `push_event` 확장(시간 이벤트 start/end dateTime, 캘린더 지정,
  생성/수정/**삭제**). `EVENTS_ENDPOINT`의 하드코딩된 `primary` 제거 → calendar_id별.
- 충돌 처리: `etag`/`updated` 비교, 기본 last-write-wins(+ 실패는 재시도 가능
  상태로 기록, 기존 `sync_status` 패턴 유지).
- 오버레이: 캘린더 목록 사이드바(`CalendarSidebar.svelte`)에서 캘린더 표시
  토글/색상, 모든 뷰에 로컬+구글 이벤트 함께 렌더.

검증: 연결→캘린더 목록 표시→구글 이벤트가 위젯에 오버레이→로컬 생성분이 구글에
반영→구글에서 수정한 게 다시 위젯에 반영(증분 동기화).

## Phase 4 — 추출(Export) 갱신

- `src-tauri/src/export.rs`: `memos` 기준 → `events` 기준으로 `fetch`/`build` 교체.
  ICS는 종일(`all_day`)/시간 이벤트 모두 `DTSTART`/`DTEND`로(현재는 all_day만).
  JSON/CSV 컬럼을 이벤트 모델로. 기존 scope(date/month/all) + 파일 저장 흐름 재사용.

검증: 시간 이벤트 포함 ICS를 구글/타 캘린더에서 정상 import, JSON/CSV 필드 확인.

---

## 핵심 결정 / 리스크

- **재동의**: 구글 스코프 상향으로 기존 토큰 재발급 필요(1회).
- **마이그레이션**: 기존 메모 보존이 핵심. Phase 1 마이그레이션에 롤백 안전장치
  (events 비었을 때만 memos 복사) 권장.
- **단일 창 채택 이유**: 자식 창 흰 화면 버그(dev) 회피 + 설정 창 동시 해결.
  별도 창이 꼭 필요하면 그 버그의 근본 원인(자식 창이 dev 서버 대신 빈 asset
  프로토콜 로드 추정)을 먼저 규명하는 별도 트랙이 선행돼야 함.
- 범위가 크므로 Phase 단위로 머지/검증. Phase 0~2(로컬+UI)만으로도 단독 가치.

## 검증 (전체 종단)

1. `npm run tauri dev` 로 실행, 위젯↔확장, 월/주/일, 설정 인라인 전환 확인.
2. 시간 이벤트 생성/수정/삭제 후 월/주/일 뷰 반영 확인.
3. `GOOGLE_CLIENT_ID` 설정 + 신규 스코프 재연결 → 캘린더 목록/오버레이/양방향
   동기화 라운드트립.
4. ICS/JSON/CSV 추출 후 외부 캘린더 import 검증.
5. Rust 단위테스트: 이벤트 CRUD/마이그레이션/ICS(시간 이벤트) — 기존
   `export.rs`/`memo.rs` 테스트 패턴 확장.
