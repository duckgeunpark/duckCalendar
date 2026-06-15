# 🦆 duckCalendar

> 바탕화면에 상시 띄워 두는 **Google Calendar 스타일 데스크톱 캘린더 위젯**
> — Tauri 2 · Svelte 5 · Rust/SQLite로 만든 로컬 우선(Local-first) 데스크톱 앱

<p>
  <img alt="Tauri" src="https://img.shields.io/badge/Tauri-2.x-24C8DB?logo=tauri&logoColor=white">
  <img alt="Svelte" src="https://img.shields.io/badge/Svelte-5%20(runes)-FF3E00?logo=svelte&logoColor=white">
  <img alt="Rust" src="https://img.shields.io/badge/Rust-stable-000000?logo=rust&logoColor=white">
  <img alt="TypeScript" src="https://img.shields.io/badge/TypeScript-5.x-3178C6?logo=typescript&logoColor=white">
  <img alt="SQLite" src="https://img.shields.io/badge/SQLite-WAL-003B57?logo=sqlite&logoColor=white">
  <img alt="Platform" src="https://img.shields.io/badge/Windows-x64-0078D6?logo=windows&logoColor=white">
</p>

---

## 한눈에 보기

**duckCalendar**는 무거운 캘린더 앱 대신, 바탕화면에 가볍게 도킹해 두고 쓰는 위젯형 캘린더입니다.
모든 데이터는 사용자 PC의 로컬 SQLite에 저장되며(서버·로그인 불필요), Google Calendar 연동은 선택 사항입니다.

- 🗓️ **월 / 주 / 일 보기**와 시간 그리드 기반 일정 관리
- 🪟 **바탕화면 고정 위젯 ↔ 확장 창** 모드 전환, 시스템 트레이 상주
- 💾 **로컬 우선** — SQLite가 단일 기준 데이터(Single Source of Truth)
- 🔄 **DB 외부 변경 자동 감지** — 파일을 직접 수정해도 화면이 실시간 갱신
- 🎨 4가지 테마 + 강조색 커스텀, 한국어/English 동시 지원

> 이 프로젝트는 **데스크톱 네이티브 셸(Tauri) + 모던 웹 프론트엔드(Svelte 5)** 조합으로
> 가벼운 단일 바이너리 앱을 설계·구현한 개인 포트폴리오 작업물입니다.

---

## 스크린샷

> _스크린샷 준비 중 — 아래 자리에 실제 화면 캡처를 추가하세요._

| 월 보기 (위젯) | 주 보기 (확장) | 설정 |
| :---: | :---: | :---: |
| _(screenshot)_ | _(screenshot)_ | _(screenshot)_ |

---

## 핵심 기능

### 보기 & 일정
- **월 / 주 / 일 보기** 전환, 월·연도 선택 팝업, 일정 있는 날 미리보기
- **주·일 시간 그리드**(1시간 = 40px): 호버한 시각 클릭으로 바로 일정 생성, 종일 영역 지원
- 일정 속성: 제목 · 시작/종료 · 종일 · 위치(토글) · 설명 · 색상 — 생성/수정/삭제 즉시 로컬 저장
- 오늘 날짜 클릭 또는 **월 달력 우클릭**으로 오늘로 이동

### 창 / 트레이 UX
- **간략보기(컴팩트 위젯) ↔ 자세히보기(확장)** 토글
- **창 모드**: 일반 · 항상 위 · 바탕화면 고정(다른 창 뒤에 머무는 위젯 모드)
- **시스템 트레이**: 좌클릭 1회 무동작 · **더블클릭으로 열기** · **우클릭으로 메뉴**(열기/숨기기/설정/종료)
- 닫기(✕)는 종료가 아니라 **바탕화면으로 도킹** — 완전 종료는 트레이 메뉴에서
- **중복 실행 방지**: 재실행 시 기존 창을 앞으로

### 모양 / 데이터
- **테마**: 다크 · 라이트 · 네이비 · 세피아 + 강조색 커스텀
- **언어**: 한국어 / English 전체 적용 · 투명도 · 글자 크기 조절
- **추출(Export)**: JSON · CSV · ICS, 범위 선택(이 날짜 / 이 달 / 전체)

---

## 기술적 도전 & 설계 결정

포트폴리오로서 가장 보여주고 싶은 부분입니다. 단순 기능 나열이 아니라, **문제를 만나 어떻게 판단했는지**를 정리했습니다.

### 1. 멀티 창 → 단일 창 위젯으로의 피벗
초기엔 메모 입력용 자식 창을 별도로 띄우는 구조였으나, 개발 모드에서 **Tauri 자식 WebView가 흰 화면으로 렌더되는 문제**를 만났습니다.
원인 추적 후, UX·안정성 양쪽에서 **단일 창 안에서 오버레이로 편집기를 띄우는 Google Calendar 스타일**로 아키텍처를 전환했습니다.
→ 창 관리 복잡도가 줄고, 위젯으로서의 일관된 경험을 확보.

### 2. WAL SQLite + 파일 워처로 외부 변경 자동 반영
DB를 외부 도구로 직접 수정해도 앱에 반영되게 하려면, 단순히 `duckCalendar.db` 하나만 감시해선 부족합니다.
SQLite를 **WAL 모드**로 열기 때문에 실제 쓰기는 `-wal` / `-journal` 사이드카 파일로 갑니다.
→ `notify` 크레이트로 **DB가 있는 디렉터리 전체를 감시**하되 `duckCalendar.db*` 파일만 필터링, 저장 시 쏟아지는 이벤트는 **300ms 디바운스**로 한 번의 `db-changed` 이벤트로 합쳐 프론트에 emit → 페이지 이동 없이 실시간 갱신.

### 3. 동기 리사이즈 콜백의 자기 데드락 회피
확장/축소 시 `WebviewWindow::set_size`가 **같은 스레드에서 `Resized` 핸들러를 동기로 재진입**합니다.
이 핸들러가 다시 DB 락을 잡으려 하면 비재진입 뮤텍스에서 **자기 데드락**으로 UI가 멈춥니다.
→ 저장된 크기를 읽은 뒤 **락을 먼저 해제하고** 리사이즈를 호출하도록 명시적으로 스코프를 분리했습니다. (`set_expanded` 참고)

### 4. 로컬 우선(Local-first) 아키텍처
SQLite를 단일 기준 데이터로 두고, 모든 CRUD를 즉시 로컬에 반영합니다.
서버·계정 없이 동작하며, 데이터는 사용자별 `%APPDATA%`에 격리 저장됩니다.

### 5. 선택적 Google Calendar 연동 (OAuth 2.0 + PKCE)
데스크톱 앱에 안전한 OAuth를 구현하기 위해 **PKCE 흐름 + 루프백 리디렉트**를 사용하고,
토큰은 코드에 두지 않고 **OS 자격 증명 저장소(`keyring`)**에 보관합니다.
민감 범위(`calendar.events`)의 Google 앱 인증 부담 때문에, 기본 배포본에서는 연동 UI를 의도적으로 꺼 두었습니다(플래그 토글로 활성화 가능).

### 6. 빌드 타임 i18n & 반응형 설정 UI
Svelte 5 runes 기반의 작은 리액티브 스토어로 언어/테마/투명도/스케일을 전역 관리하고,
`ui.lang`을 마크업에서 직접 읽어 **언어 전환 시 전체가 즉시 리렌더**되도록 했습니다.

---

## 아키텍처

```
duckCalendar/
├─ src/                    # 프론트엔드 (Svelte 5 + TS)
│  ├─ App.svelte           # 셸: 타이틀바 · 보기 라우팅 · 백엔드 이벤트 수신
│  └─ lib/
│     ├─ Calendar.svelte   # 월 보기
│     ├─ TimeGrid.svelte   # 주/일 시간 그리드
│     ├─ DayView.svelte    # 일 목록 보기
│     ├─ EventEditor.svelte# 일정 편집 오버레이
│     ├─ Settings.svelte   # 카드형 반응형 설정
│     ├─ store.svelte.ts   # runes 기반 전역 UI 상태 + 이벤트 버스
│     ├─ api.ts            # Tauri invoke 래퍼
│     └─ i18n.ts           # 한/영 사전 + 리액티브 번역 헬퍼
└─ src-tauri/              # 백엔드 (Rust)
   └─ src/
      ├─ lib.rs            # 앱 부트스트랩 · 창/트레이 관리 · DB 파일 워처
      ├─ db.rs             # SQLite 연결 · 마이그레이션(WAL)
      ├─ event.rs          # 캘린더/이벤트 CRUD 커맨드
      ├─ settings.rs       # app_settings 키/값 영속화
      ├─ export.rs         # JSON/CSV/ICS 추출
      └─ google/           # OAuth(PKCE) · REST 동기화(선택)
```

**데이터 흐름:** Svelte UI → `invoke`(api.ts) → Rust 커맨드 → SQLite.
외부 DB 변경은 Rust 파일 워처 → `db-changed` 이벤트 → 프론트의 작은 이벤트 버스(`bumpMemos`) → 보이는 뷰 재조회.

---

## 기술 스택

| 영역 | 사용 기술 |
| --- | --- |
| 셸 / 번들 | **Tauri 2.x** (Rust 코어 + WebView2, 단일 바이너리) |
| 프론트엔드 | **Svelte 5 (runes)** · TypeScript · Vite |
| 백엔드 / DB | **Rust** · **SQLite**(`rusqlite`, bundled, WAL) |
| 파일 감시 | `notify` (DB 외부 변경 자동 감지) |
| 연동(선택) | Google Calendar REST(`reqwest`), 토큰은 `keyring` |
| 플러그인 | `single-instance`(중복 실행 방지) · `dialog` · `opener` |

---

## 시작하기

**사전 요구 사항:** Rust(stable), Node.js 18+, WebView2 런타임, Visual Studio C++ Build Tools(MSVC)

```powershell
npm install            # 프론트엔드 의존성
npm run tauri dev      # 개발 모드 (Rust 백엔드 + Vite 핫리로드)
npm run tauri build    # 배포 번들 생성
```

**배포 산출물** (`src-tauri/target/release/`):

- 단독 실행 파일: `duck-calendar.exe`
- NSIS 설치 관리자: `bundle/nsis/duckCalendar_<버전>_x64-setup.exe`
- MSI 패키지: `bundle/msi/duckCalendar_<버전>_x64_en-US.msi`

> 배포 시 `*-setup.exe` 하나만 전달하면 됩니다(필요한 모든 것이 패키징됨). 64비트 Windows 전용이며,
> 코드 서명 미적용 상태라 SmartScreen에서 "추가 정보 → 실행" 안내가 필요합니다.

---

## 데이터 위치

- 로컬 DB: `%APPDATA%/com.duckcalendar.app/duckCalendar.db`
- OAuth 토큰(연동 사용 시): Windows 자격 증명 관리자(`keyring`), 연결 해제 시 삭제

---

<details>
<summary><b>Google Calendar 연동 설정 (선택 · 고급)</b></summary>

연동을 사용하려면 본인 소유의 OAuth 클라이언트가 필요하고, `src/lib/Settings.svelte`의 `GOOGLE_ENABLED`를 `true`로 바꿔야 합니다.

1. [Google Cloud Console](https://console.cloud.google.com/) → 프로젝트 생성
2. **Google Calendar API** 사용 설정
3. **OAuth 동의 화면** 구성(테스트 단계라면 테스트 사용자에 본인 계정 추가)
4. **사용자 인증 정보 → OAuth 클라이언트 ID → 데스크톱 앱** 생성(루프백 리디렉트 자동 허용)
5. 발급된 **클라이언트 ID**를 설정 화면에 붙여넣고 저장(로컬 DB에 보관)

- 클라이언트 ID는 앱의 식별자이며 사용자 계정이 아닙니다.
- "테스트" 상태에서는 등록된 테스트 사용자만 연결 가능하고 토큰이 약 7일 후 만료됩니다.
- 불특정 다수 배포에는 OAuth 동의 화면을 프로덕션 게시 + Google 인증이 필요합니다(`calendar.events`는 민감 범위).
- 요청 권한은 `https://www.googleapis.com/auth/calendar.events` 하나로 최소화되어 있습니다.

</details>

---

## 설계 문서

초기 기획·설계 초안: [`documents/calendar_widget_design_draft_v1.md`](documents/calendar_widget_design_draft_v1.md)
</content>
</invoke>
