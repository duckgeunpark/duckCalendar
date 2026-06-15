# duckCalendar

Windows 데스크톱 캘린더 위젯 — 바탕화면에 상시 띄워 두는 단일 창(Google Calendar 스타일) 캘린더와 일정 관리.
**로컬 우선(Local-first)** 구조로, 로컬 SQLite가 단일 기준 데이터입니다. (Google Calendar 연동은 선택 기능이며 현재 UI에서는 비활성화되어 있습니다 — 아래 참고.)

설계서: [`documents/calendar_widget_design_draft_v1.md`](documents/calendar_widget_design_draft_v1.md)

## 기능

### 보기
- **월 / 주 / 일 보기** 전환 (자세히보기에서 상단 보기바로 선택)
- **월 보기**: 현재 월 달력, 이전/다음 달 이동, 월/연도 선택 팝업, 일정 있는 날에 미리보기 표시
- **주 / 일 보기**: 1시간 = 40px 시간 그리드, 종일 일정 영역
  - 마우스를 올린 시간대를 **호버로 표시**, 클릭하면 그 시각으로 일정 생성
  - **종일 칸 클릭** 시 종일 일정 생성
- 우측 상단의 오늘 날짜를 누르거나 **월 달력에서 우클릭**하면 오늘로 이동
- 주/일 보기에서 빠져나갈 때(우클릭) 월 보기로 복귀

### 일정
- 제목 · 시작/종료 시각 · **종일** · **위치(옵션 토글)** · 설명 · 색상
- 로컬 SQLite 즉시 저장(생성·수정·삭제)
- 편집 화면: 한 줄 헤더(목록 · 날짜 · 저장/취소/삭제)로 입력 영역을 넓게 사용

### 창 / 트레이 동작
- **간략보기(컴팩트 위젯) ↔ 자세히보기(확장)** 토글 (상단 ⊞/⊟ 버튼 또는 설정)
- **창 모드**: 일반 · 항상 위 · 바탕화면 고정(위젯; 다른 창 뒤·창 버튼 숨김)
- **시스템 트레이 메뉴**: 열기 · 숨기기 · 설정 · 종료
  - 트레이 아이콘 **싱글/더블 클릭**으로 열기
  - 열기/설정 시 **일반 모드로 전환 + 창을 앞으로** 가져옴(설정은 설정 화면까지 표시)
- 창 **닫기(✕) → 바탕화면 고정으로 도킹**(종료 아님). 완전 종료는 트레이의 **종료**
- **중복 실행 방지**: 이미 실행 중이면 새 창 대신 기존 창을 띄움

### 모양 / 언어
- **테마**: 다크 · 라이트 · 네이비 · 세피아 + **강조색 커스텀**(색상 선택기)
- **언어**: 한국어 / English (앱 전체 적용)
- **투명도** · **글자 크기** 조절
- 카드 기반 반응형 설정 화면(창 너비에 따라 카드가 재배치)

### 데이터
- **추출(Export)**: JSON · CSV · ICS, 범위 선택(이 날짜 / 이 달 / 전체)

### Google Calendar 연동 (현재 비활성화)
- OAuth 2.0 + PKCE 흐름, 토큰은 OS 자격 증명 저장소(`keyring`)에 보관하는 코드가 구현되어 있습니다.
- **현재 설정 UI에서는 숨겨져 있습니다**(`src/lib/Settings.svelte`의 `GOOGLE_ENABLED = false`).
  불특정 다수 배포 시 Google의 OAuth 앱 인증이 필요하기 때문입니다(아래 "Google Calendar 연동 설정" 참고).

## 기술 스택

- **셸/번들**: Tauri 2.x (Rust 코어 + WebView2)
- **프론트엔드**: Svelte 5 (runes) + TypeScript + Vite
- **로컬 DB**: SQLite (`rusqlite`, bundled)
- **연동(선택)**: Google Calendar REST (`reqwest`), 토큰은 OS 자격 증명 저장소(`keyring`)
- **기타 플러그인**: `tauri-plugin-single-instance`(중복 실행 방지), `tauri-plugin-dialog`, `tauri-plugin-opener`

## 사전 요구 사항 (개발/빌드)

1. **Rust** (stable) — https://rustup.rs (`cargo`, `rustc`)
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

생성물:

- 단독 실행 파일: `src-tauri/target/release/duck-calendar.exe`
- NSIS 설치 관리자: `src-tauri/target/release/bundle/nsis/duckCalendar_<버전>_x64-setup.exe`
- MSI 패키지: `src-tauri/target/release/bundle/msi/duckCalendar_<버전>_x64_en-US.msi`

> **아이콘**: 아이콘을 바꾸려면 1024x1024 PNG로 `npm run tauri icon path/to/icon.png` 를 실행하면
> `src-tauri/icons/` 에 플랫폼별 아이콘이 생성됩니다.

## 배포 시 참고

- **하나의 설치 파일(`*-setup.exe` 권장)만** 전달하면 됩니다. 실행에 필요한 모든 것이 패키징되어 있습니다.
- **코드 서명 미적용**: SmartScreen에서 "알 수 없는 게시자" 경고가 뜹니다. 사용자에게 **"추가 정보 → 실행"** 을 안내하세요.
  경고 제거에는 코드 서명 인증서(EV/OV 또는 Azure Trusted Signing)가 필요합니다.
- **64비트 Windows 전용**(x64). 데이터는 사용자별 로컬(`%APPDATA%`)에 저장되어 사용자 간 섞이지 않습니다.

## Google Calendar 연동 설정 (선택 · 고급)

연동을 사용하려면 본인 소유의 OAuth 클라이언트가 필요하고, `src/lib/Settings.svelte`의 `GOOGLE_ENABLED` 를 `true` 로 바꿔야 합니다.

1. [Google Cloud Console](https://console.cloud.google.com/) → 프로젝트 생성
2. **Google Calendar API** 사용 설정
3. **OAuth 동의 화면** 구성 (테스트 단계라면 테스트 사용자에 본인 계정 추가)
4. **사용자 인증 정보 → OAuth 클라이언트 ID → 애플리케이션 유형: 데스크톱 앱** 생성 (루프백 리디렉트를 자동 허용)
5. 발급된 **클라이언트 ID**를 설정 화면의 입력칸에 붙여넣고 저장 (로컬 DB에 보관)

> - 클라이언트 ID는 **앱의 식별자**이며 사용자 계정이 아닙니다(모든 사용자 공통, 한 번만 발급).
> - "테스트" 상태에서는 등록된 테스트 사용자만 연결 가능하고 토큰이 약 7일 후 만료됩니다.
> - **불특정 다수 배포**에는 OAuth 동의 화면을 **프로덕션으로 게시 + Google 인증**이 필요합니다
>   (`calendar.events`는 민감 범위). 그래서 기본 배포본에서는 연동 UI를 꺼 두었습니다.
> - 요청 권한(scope)은 `https://www.googleapis.com/auth/calendar.events` 하나로 최소화되어 있습니다.

## 데이터 위치

- 로컬 DB: `%APPDATA%/com.duckcalendar.app/duckCalendar.db`
- OAuth 토큰(연동 사용 시): Windows 자격 증명 관리자(`keyring`), 연결 해제 시 삭제됩니다.
