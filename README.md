# duckCalendar

Windows 데스크톱 캘린더 위젯 — 바탕화면에서 상시 확인하는 월간 달력과 날짜별 메모.
**로컬 우선(Local-first)** 구조로, 로컬 SQLite가 단일 기준 데이터이며 Google Calendar 연동은 선택 기능입니다.

설계서: [`documents/calendar_widget_design_draft_v1.md`](documents/calendar_widget_design_draft_v1.md)

## 기능

- 시스템 시간 기준 현재 월 달력 표시, 이전/다음 달 이동, 메모 있는 날 강조
- 날짜별 메모 생성·조회·수정·삭제 (로컬 SQLite 즉시 저장)
- JSON / CSV / ICS 추출 (날짜·월·전체 범위)
- 선택적 Google Calendar 연동 (OAuth 2.0 + PKCE, 사용자가 지정한 메모만 반영)
- 프레임리스 위젯 창 + 시스템 트레이

## 기술 스택

- **셸/번들**: Tauri 2.x (Rust 코어 + WebView2)
- **프론트엔드**: Svelte 5 + TypeScript + Vite
- **로컬 DB**: SQLite (`rusqlite`, bundled)
- **연동**: Google Calendar REST (`reqwest`), 토큰은 OS 자격 증명 저장소(`keyring`)

## 사전 요구 사항 (개발/빌드)

현재 머신에는 아래 툴체인이 설치되어 있지 않습니다. 빌드 전에 설치하세요.

1. **Rust** (stable) — https://rustup.rs (`rustup` → `cargo`, `rustc`)
2. **Node.js** 18+ (npm 포함) — https://nodejs.org
3. **WebView2 런타임** — Windows 11에는 기본 포함. 없으면 Microsoft에서 설치.
4. **Visual Studio C++ Build Tools** (MSVC) — Rust Windows 빌드에 필요.

설치 확인:

```powershell
rustc --version
node --version
```

## 실행

```powershell
npm install            # 프론트엔드 의존성
npm run tauri dev      # 개발 모드 (Rust 백엔드 + Vite 핫리로드)
```

배포 빌드:

```powershell
npm run tauri build    # 설치 번들 생성 (src-tauri/target/release/bundle)
```

> **아이콘**: 최초 빌드 전 앱 아이콘이 필요합니다. 1024x1024 PNG를 준비한 뒤
> `npm run tauri icon path/to/icon.png` 를 실행하면 `src-tauri/icons/` 에 플랫폼별 아이콘이 생성됩니다.

## Google Calendar 연동 설정 (선택)

연동을 사용하려면 본인 소유의 OAuth 클라이언트가 필요합니다.

1. [Google Cloud Console](https://console.cloud.google.com/) → 프로젝트 생성
2. **Google Calendar API** 사용 설정
3. **OAuth 동의 화면** 구성 (테스트 사용자에 본인 계정 추가)
4. **사용자 인증 정보 → OAuth 클라이언트 ID → 애플리케이션 유형: 데스크톱 앱** 생성
5. 발급된 **클라이언트 ID**를 환경 변수로 주입 (데스크톱 앱은 PKCE를 사용하므로 시크릿은 선택):

```powershell
$env:GOOGLE_CLIENT_ID = "xxxxxxxx.apps.googleusercontent.com"
# (클라이언트 시크릿이 있다면) $env:GOOGLE_CLIENT_SECRET = "..."
npm run tauri dev
```

클라이언트 ID가 없으면 연동 기능만 비활성화되고 달력·메모·추출 등 나머지 기능은 정상 동작합니다.
요청 권한(scope)은 `https://www.googleapis.com/auth/calendar.events` (이벤트 읽기/쓰기) 하나로 최소화되어 있습니다.

## 데이터 위치

- 로컬 DB: `%APPDATA%/com.duckcalendar.app/duckCalendar.db`
- OAuth 토큰: Windows 자격 증명 관리자 (`keyring`), 연결 해제 시 삭제됩니다.
