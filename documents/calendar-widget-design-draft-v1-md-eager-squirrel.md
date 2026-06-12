# duckCalendar 구현 계획 — 데스크톱 캘린더 위젯 (Tauri 2.x)

## Context (배경)

`documents/calendar_widget_design_draft_v1.md` 설계서가 정의한 **Windows 데스크톱 캘린더 위젯**을 구현한다.
핵심 요구는 (1) 바탕화면 상시 표시 월간 달력, (2) 날짜별 메모 로컬 CRUD, (3) JSON/CSV/ICS 추출,
(4) 선택적 Google Calendar 연동이며, 전체가 **로컬 우선(Local-first)** 구조를 따른다 — 로컬 SQLite가
단일 기준 데이터(source of truth)이고 외부 동기화는 부가 계층이다.

현재 저장소는 빈 상태(README, .gitignore, 설계 문서만 존재)이며 **재사용할 기존 코드는 없다**.
사용자 결정에 따라 **Tauri 2.x (Rust 코어 + 웹 프론트엔드)** 스택으로 진행하며, '경량성' 비기능 요구
(작은 설치 크기·낮은 메모리)에 부합한다. 이번 구현에서 **Google OAuth 실연동까지 전부** 포함한다.

> 주의: 현재 `.gitignore`는 C++/CMake/vcpkg 용이므로 Rust/Node 용으로 교체해야 한다.

---

## 기술 스택

| 영역 | 선택 | 비고 |
|------|------|------|
| 셸/번들 | Tauri 2.x | WebView2 재사용, 번들 ~3–10MB |
| 백엔드 | Rust | IPC command로 비즈니스 로직 처리 |
| 프론트 | Svelte + TypeScript + Vite | 컴파일 후 경량 JS, 달력 반응형 UI에 적합 (대안: vanilla TS) |
| 로컬 DB | SQLite (`rusqlite`, bundled) | 설계서 권장 저장소 |
| 날짜 | `chrono` | 시스템 시간 기준 월 계산 |
| ICS | `icalendar` crate | VEVENT 생성 |
| OAuth | `oauth2` crate + loopback redirect | Authorization Code + PKCE |
| Google API | `reqwest` (rustls) | Calendar events REST |
| 토큰 저장 | `keyring` crate | Windows 자격 증명 관리자에 안전 저장 |
| 기타 | `serde`/`serde_json`, `uuid`, `tiny_http`(loopback) | |
| Tauri 플러그인 | `opener`(브라우저), `dialog`(파일 저장), `tray-icon` | |

---

## 프로젝트 구조 (생성 대상)

```
duckCalendar/
├─ src/                      # 프론트엔드 (Svelte + TS)
│  ├─ main.ts, App.svelte
│  ├─ lib/Calendar.svelte    # 월간 달력 그리드 + 이전/다음 이동
│  ├─ lib/MemoPanel.svelte   # 선택 날짜 메모 목록·편집·삭제
│  ├─ lib/Settings.svelte    # Google 연결/해제, 추출 버튼
│  └─ lib/api.ts             # invoke() 래퍼 (Tauri command 호출)
├─ src-tauri/
│  ├─ src/
│  │  ├─ main.rs / lib.rs    # Tauri 빌더, command 등록, 트레이
│  │  ├─ db.rs               # SQLite 연결·마이그레이션
│  │  ├─ memo.rs             # 메모 CRUD command
│  │  ├─ export.rs           # JSON/CSV/ICS 변환
│  │  ├─ settings.rs         # app_settings 읽기/쓰기 (창 위치 등)
│  │  └─ google/
│  │     ├─ oauth.rs         # PKCE + loopback + keyring 토큰
│  │     └─ sync.rs          # 선택 메모 → Calendar event 반영
│  ├─ tauri.conf.json        # 프레임리스 창, 트레이, 권한
│  └─ Cargo.toml
├─ package.json, vite.config.ts, tsconfig.json
└─ .gitignore               # Rust/Node 용으로 교체
```

---

## 데이터 모델 (SQLite — 설계서 4절 그대로 매핑)

```sql
CREATE TABLE memos (
  memo_id           TEXT PRIMARY KEY,      -- uuid
  target_date       TEXT NOT NULL,         -- 'YYYY-MM-DD'
  title             TEXT NOT NULL,
  content           TEXT NOT NULL DEFAULT '',
  created_at        TEXT NOT NULL,         -- ISO8601
  updated_at        TEXT NOT NULL,
  is_calendar_event INTEGER NOT NULL DEFAULT 0,  -- 일정성 데이터 여부
  sync_enabled      INTEGER NOT NULL DEFAULT 0   -- Google 반영 대상 여부
);
CREATE INDEX idx_memos_date ON memos(target_date);

CREATE TABLE google_sync_map (
  memo_id           TEXT PRIMARY KEY REFERENCES memos(memo_id) ON DELETE CASCADE,
  google_event_id   TEXT,
  sync_status       TEXT NOT NULL DEFAULT 'pending', -- pending|synced|failed
  last_synced_at    TEXT,
  last_error_message TEXT
);

CREATE TABLE app_settings (
  setting_key   TEXT PRIMARY KEY,
  setting_value TEXT NOT NULL
);
```
DB 위치: `app_data_dir()/duckCalendar.db` (Tauri 경로 API). OAuth 토큰은 DB가 아닌 keyring에 저장.

---

## 구현 단계

### 1단계 — 스캐폴드 & 인프라
- `create-tauri-app`(Svelte-TS 템플릿)으로 초기화, `.gitignore`를 Rust/Node 용으로 교체.
- `db.rs`: 앱 시작 시 DB 열고 위 스키마 마이그레이션(없으면 생성), `PRAGMA foreign_keys=ON`.
- 공통 결과 타입과 에러 매핑(`Result<T, String>`) 정립.

### 2단계 — 달력 + 메모 CRUD (로컬, 핵심)
- Rust command:
  - `list_memo_dates(year, month) -> Vec<String>` (메모 존재 날짜 → 달력 표시 구분)
  - `list_memos_by_date(date) -> Vec<Memo>`
  - `create_memo / update_memo / delete_memo`
- 프론트 `Calendar.svelte`: `chrono`로 받은 현재 연월 그리드 렌더, 이전/다음 이동, 메모 있는 날 강조.
- `MemoPanel.svelte`: 날짜 클릭 → 메모 목록·상세, 생성/수정/삭제. 저장 즉시 DB 반영.
- `is_calendar_event`/`sync_enabled` 토글 UI(일정으로 지정 / Google 반영 대상).

### 3단계 — 데이터 추출 (`export.rs`)
- `export_json(scope)` / `export_csv(scope)` / `export_ics(scope)` — scope = 날짜 | 월 | 전체.
- ICS: `is_calendar_event = 1` 메모를 `icalendar` crate로 VEVENT 변환(all-day DTSTART/DTEND).
- `dialog` 플러그인으로 저장 경로 선택 후 파일 쓰기.

### 4단계 — 위젯 창 & 설정 지속성
- `tauri.conf.json`: `decorations:false`, `skipTaskbar:true`, 적당한 소형 크기, 시스템 트레이 아이콘(표시/숨김/종료).
- 창 위치·마지막 표시 월·테마를 `app_settings`에 저장/복원(`settings.rs`). 종료 시 위치 기록, 시작 시 복원.

### 5단계 — Google Calendar 선택 연동 (`google/`)
- **사전 준비(사용자 작업)**: Google Cloud Console에서 **Desktop app** 유형 OAuth 클라이언트 ID 발급 →
  `client_id`(데스크톱은 secret 미보안 → **PKCE** 사용). 빌드 설정/환경변수로 주입.
- `oauth.rs` 연동 흐름(설계서 3절):
  1. `google_connect` 호출 → 로컬 loopback 서버(`tiny_http`) 기동 + PKCE 생성.
  2. `opener`로 시스템 기본 브라우저에서 Google 동의 화면 오픈.
  3. 사용자 동의 → loopback으로 `code` 수신 → 토큰 교환.
  4. access/refresh 토큰을 **keyring**에 저장, 연동 상태 활성화.
- **최소 권한**: scope = `https://www.googleapis.com/auth/calendar.events` (이벤트 읽기/쓰기만).
- `sync.rs`: `sync_enabled = 1` 메모만 `reqwest`로 events.insert/update,
  `google_sync_map`에 event_id·status 기록. 토큰 만료 시 refresh.
- `google_disconnect`: keyring 토큰 삭제, 상태 비활성화 (로컬 메모는 유지).
- **실패 처리(설계서 6절)**: 로그인 실패/권한 거부/토큰 만료/네트워크 오류 구분 →
  `sync_status='failed'` + `last_error_message` 기록, 재시도 가능 상태 유지. 로컬 기능은 영향 없음.

---

## 주요 설계 원칙 (설계서 준수)
- 로컬 우선: 모든 command는 로컬 DB 먼저 갱신, Google 반영은 별도/비동기. 동기화 실패가 로컬 손실로 이어지지 않음.
- 자동 업로드 금지: 사용자가 `sync_enabled` 지정한 항목만 Google에 반영.
- 최소 권한 + 토큰 안전 저장(keyring), 연결 해제 시 토큰 제거.
- Google 미연동 상태에서도 달력·메모·추출 전부 정상 동작.

---

## 검증 (Verification)
1. **개발 실행**: `npm install` → `cargo tauri dev` 로 위젯 기동. 프레임리스 창 + 트레이 표시 확인.
2. **로컬 핵심 흐름(수동)**:
   - 현재 월이 시스템 시간 기준으로 표시되는지, 이전/다음 이동 동작.
   - 날짜 선택 → 메모 생성/조회/수정/삭제, 메모 있는 날 시각 구분.
   - 앱 재시작 후 창 위치·메모 복원(안정성/지속성).
3. **추출**: JSON/CSV/ICS 각각 저장 → 파일 내용 확인. 생성된 `.ics`를 Google Calendar 가져오기로 검증.
4. **Rust 단위 테스트**: `export.rs`(ICS VEVENT 형식, JSON 직렬화), `db.rs`(CRUD 왕복) `cargo test`.
5. **Google 연동(수동)**: `google_connect` → 브라우저 동의 → 토큰 저장 확인 →
   `sync_enabled` 메모 1건 동기화 → 실제 Google Calendar에 이벤트 생성 확인 →
   네트워크 차단 시 `failed` 기록·로컬 정상 동작 확인 → `google_disconnect` 후 토큰 제거 확인.

---

## 열린 항목 / 후속(설계서 제외 범위)
반복 일정, 다중 계정, 알림/리마인더, ICS 가져오기, 팀 공유는 이번 범위 제외(설계서 명시).
Google OAuth 클라이언트 ID는 사용자가 발급해 주입해야 하며, 미주입 시 연동 기능만 비활성화되고
나머지 기능은 정상 동작하도록 가드한다.
