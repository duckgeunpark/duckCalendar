<script lang="ts">
  import { onMount } from "svelte";
  import { emit } from "@tauri-apps/api/event";
  import {
    googleStatus,
    googleConnect,
    googleDisconnect,
    syncSelected,
    exportToFile,
    getSetting,
    setSetting,
    setWindowMode,
    setExpanded,
    setGoogleClientId,
  } from "./api";
  import type { ExportFormat, ExportScope, GoogleStatus } from "./types";
  import { ui } from "./store.svelte";
  import { t } from "./i18n";

  interface Props {
    selectedDate: string; // 'YYYY-MM-DD'
  }
  let { selectedDate }: Props = $props();

  let status = $state<GoogleStatus>({ configured: false, connected: false });
  let busy = $state(false);
  let message = $state("");

  let scope = $state<ExportScope>("month");
  let clientId = $state("");
  let accentColor = $state("#ffc400");

  // Google 연동 기능 노출 여부. 공개 배포 + OAuth 인증을 마치면 true 로.
  const GOOGLE_ENABLED = false;

  $effect(() => {
    if (GOOGLE_ENABLED) refreshStatus();
  });

  // Load current appearance into the shared store for this window.
  onMount(async () => {
    const mode = await getSetting("window_mode");
    if (mode === "top" || mode === "desktop" || mode === "normal") ui.windowMode = mode;
    const op = await getSetting("opacity");
    const n = op ? Number(op) : NaN;
    if (!Number.isNaN(n) && n >= 0.3 && n <= 1) ui.opacity = n;
    const sc = await getSetting("ui_scale");
    const s = sc ? Number(sc) : NaN;
    if (!Number.isNaN(s) && s >= 0.7 && s <= 1.5) ui.scale = s;
    clientId = (await getSetting("google_client_id")) ?? "";
    const th = await getSetting("theme");
    if (th === "dark" || th === "light" || th === "navy" || th === "sepia") ui.theme = th;
    ui.accent = (await getSetting("accent")) ?? "";
    accentColor = ui.accent || "#ffc400";
    const lg = await getSetting("lang");
    if (lg === "ko" || lg === "en") ui.lang = lg;
  });

  function applyTheme() {
    setSetting("theme", ui.theme).catch((e) => (message = String(e)));
  }
  function applyLang() {
    setSetting("lang", ui.lang).catch((e) => (message = String(e)));
  }

  let accentTimer: ReturnType<typeof setTimeout> | undefined;
  function onAccentInput() {
    ui.accent = accentColor;
    clearTimeout(accentTimer);
    accentTimer = setTimeout(() => {
      setSetting("accent", ui.accent).catch((e) => (message = String(e)));
    }, 200);
  }
  function resetAccent() {
    ui.accent = "";
    accentColor = "#ffc400";
    setSetting("accent", "").catch((e) => (message = String(e)));
  }

  async function saveClientId() {
    busy = true;
    try {
      await setGoogleClientId(clientId.trim());
      message = clientId.trim() ? "클라이언트 ID를 저장했습니다." : "클라이언트 ID를 비웠습니다.";
      await refreshStatus();
    } catch (e) {
      message = String(e);
    } finally {
      busy = false;
    }
  }

  // Live-preview the font scale in this settings window too.
  $effect(() => {
    document.documentElement.style.setProperty("zoom", String(ui.scale));
  });

  // Tell the main widget to update its appearance live.
  function broadcastAppearance() {
    emit("appearance-changed", {
      windowMode: ui.windowMode,
      opacity: ui.opacity,
      scale: ui.scale,
    });
  }

  // 간략보기(컴팩트 위젯) ↔ 자세히보기(확장 레이아웃). 메인 위젯과 같은 상태를
  // 공유하므로 ui.expanded만 바꾸면 화면이 따라오고, 창 크기는 setExpanded로 조정.
  async function applyExpanded(expanded: boolean) {
    if (ui.expanded === expanded) return;
    ui.expanded = expanded;
    try {
      await setExpanded(expanded);
    } catch (e) {
      message = String(e);
    }
  }

  async function applyWindowMode() {
    try {
      await setWindowMode(ui.windowMode);
      broadcastAppearance();
      message = "창 모드를 변경했습니다.";
    } catch (e) {
      message = String(e);
    }
  }

  // Apply opacity live to the main window; debounce the DB persist.
  let opacityTimer: ReturnType<typeof setTimeout> | undefined;
  function onOpacityInput() {
    broadcastAppearance();
    clearTimeout(opacityTimer);
    opacityTimer = setTimeout(() => {
      setSetting("opacity", String(ui.opacity)).catch((e) => (message = String(e)));
    }, 200);
  }

  let scaleTimer: ReturnType<typeof setTimeout> | undefined;
  function onScaleInput() {
    broadcastAppearance();
    clearTimeout(scaleTimer);
    scaleTimer = setTimeout(() => {
      setSetting("ui_scale", String(ui.scale)).catch((e) => (message = String(e)));
    }, 200);
  }

  async function refreshStatus() {
    try {
      status = await googleStatus();
    } catch (e) {
      message = String(e);
    }
  }

  function scopeValue(s: ExportScope): string {
    if (s === "date") return selectedDate;
    if (s === "month") return selectedDate.slice(0, 7);
    return "";
  }

  async function connect() {
    busy = true;
    message = "브라우저에서 로그인 및 동의를 완료하세요…";
    try {
      await googleConnect();
      message = "연결되었습니다.";
      await refreshStatus();
    } catch (e) {
      message = "연결 실패: " + String(e);
    } finally {
      busy = false;
    }
  }

  async function disconnect() {
    busy = true;
    try {
      await googleDisconnect();
      message = "연결이 해제되었습니다.";
      await refreshStatus();
    } catch (e) {
      message = String(e);
    } finally {
      busy = false;
    }
  }

  async function syncAll() {
    busy = true;
    message = "동기화 중…";
    try {
      const r = await syncSelected();
      message = `동기화 완료: 성공 ${r.synced}건, 실패 ${r.failed}건`;
      if (r.errors.length) message += "\n" + r.errors.join("\n");
    } catch (e) {
      message = String(e);
    } finally {
      busy = false;
    }
  }

  async function doExport(format: ExportFormat) {
    busy = true;
    message = "";
    try {
      const path = await exportToFile(scope, scopeValue(scope), format);
      message = path ? `저장됨: ${path}` : "취소되었습니다.";
    } catch (e) {
      message = "추출 실패: " + String(e);
    } finally {
      busy = false;
    }
  }
</script>

<section class="settings">
  <div class="head">
    <h3>{t("settings")}</h3>
  </div>

  <div class="cards-row">
    <!-- 화면 설정 -->
    <div class="card">
      <div class="card-head"><span class="ic">🖥️</span>{t("displayHead")}</div>

      <h4>{t("viewModeHead")}</h4>
      <div class="row">
        <label class="chk"><input type="radio" name="viewmode" checked={!ui.expanded} onchange={() => applyExpanded(false)} /> {t("expandCompact")}</label>
        <label class="chk"><input type="radio" name="viewmode" checked={ui.expanded} onchange={() => applyExpanded(true)} /> {t("expandDetailed")}</label>
      </div>
      <p class="hint">{t("viewModeHint")}</p>

      <h4 class="sub">{t("windowModeHead")}</h4>
      <div class="row">
        <label class="chk"><input type="radio" name="winmode" value="normal" bind:group={ui.windowMode} onchange={applyWindowMode} /> {t("wmNormal")}</label>
        <label class="chk"><input type="radio" name="winmode" value="top" bind:group={ui.windowMode} onchange={applyWindowMode} /> {t("wmTop")}</label>
        <label class="chk"><input type="radio" name="winmode" value="desktop" bind:group={ui.windowMode} onchange={applyWindowMode} /> {t("wmDesktop")}</label>
      </div>
      <p class="hint">{t("windowModeHint")}</p>
    </div>

    <!-- 가독성 및 효과 -->
    <div class="card">
      <div class="card-head"><span class="ic">👓</span>{t("readabilityHead")}</div>

      <h4>{t("opacity")}</h4>
      <div class="row opacity">
        <input type="range" min="0.3" max="1" step="0.05" bind:value={ui.opacity} oninput={onOpacityInput} />
        <span class="opacity-val">{Math.round(ui.opacity * 100)}%</span>
      </div>

      <h4 class="sub">{t("fontSize")}</h4>
      <div class="row opacity">
        <input type="range" min="0.8" max="1.4" step="0.05" bind:value={ui.scale} oninput={onScaleInput} />
        <span class="opacity-val">{Math.round(ui.scale * 100)}%</span>
      </div>
    </div>
  </div>

  <!-- 시스템 및 데이터 설정 -->
  <div class="card">
    <div class="card-head"><span class="ic">☁️</span>{t("systemHead")}</div>

    <div class="subcards">
      <!-- 언어 설정 -->
      <div class="subcard">
        <div class="card-head sub-head"><span class="ic">🌐</span>{t("languageCardHead")}</div>
        <h4>{t("languageHead")}</h4>
        <div class="col">
          <label class="chk"><input type="radio" name="lang" value="ko" bind:group={ui.lang} onchange={applyLang} /> {t("langKo")}</label>
          <label class="chk"><input type="radio" name="lang" value="en" bind:group={ui.lang} onchange={applyLang} /> {t("langEn")}</label>
        </div>
      </div>

      <!-- 스타일 설정 -->
      <div class="subcard">
        <div class="card-head sub-head"><span class="ic">🎨</span>{t("styleHead")}</div>
        <h4>{t("themeHead")}</h4>
        <div class="row">
          <label class="chk"><input type="radio" name="theme" value="dark" bind:group={ui.theme} onchange={applyTheme} /> {t("themeDark")}</label>
          <label class="chk"><input type="radio" name="theme" value="light" bind:group={ui.theme} onchange={applyTheme} /> {t("themeLight")}</label>
          <label class="chk"><input type="radio" name="theme" value="navy" bind:group={ui.theme} onchange={applyTheme} /> {t("themeNavy")}</label>
          <label class="chk"><input type="radio" name="theme" value="sepia" bind:group={ui.theme} onchange={applyTheme} /> {t("themeSepia")}</label>
        </div>
        <h4 class="sub">{t("accentHead")}</h4>
        <div class="row opacity">
          <input class="swatch" type="color" bind:value={accentColor} oninput={onAccentInput} />
          <span class="opacity-val grow">{ui.accent ? accentColor : t("accentDefault")}</span>
          <button class="ghost" onclick={resetAccent} disabled={!ui.accent}>{t("accentReset")}</button>
        </div>
      </div>

      <!-- 데이터 추출 -->
      <div class="subcard">
        <div class="card-head sub-head"><span class="ic">📤</span>{t("exportHead")}</div>
        <h4>{t("exportScopeHead")}</h4>
        <div class="row">
          <label class="chk"><input type="radio" name="scope" value="date" bind:group={scope} /> {t("scopeDate")}</label>
          <label class="chk"><input type="radio" name="scope" value="month" bind:group={scope} /> {t("scopeMonth")}</label>
          <label class="chk"><input type="radio" name="scope" value="all" bind:group={scope} /> {t("scopeAll")}</label>
        </div>
        <h4 class="sub">{t("exportFormatHead")}</h4>
        <div class="row">
          <button onclick={() => doExport("json")} disabled={busy}>JSON</button>
          <button onclick={() => doExport("csv")} disabled={busy}>CSV</button>
          <button onclick={() => doExport("ics")} disabled={busy}>ICS</button>
        </div>
      </div>
    </div>
    <p class="hint">{t("exportHint")}</p>
  </div>

  <!-- Google 연동은 공개 배포(OAuth 인증) 전까지 숨김. GOOGLE_ENABLED=true 로 되살릴 수 있음. -->
  {#if GOOGLE_ENABLED}
    <div class="block">
      <h4>Google Calendar 연동</h4>
      <label class="hint" for="gcid">Google OAuth 클라이언트 ID (데스크톱 앱 유형)</label>
      <div class="row">
        <input id="gcid" class="grow" placeholder="xxxx.apps.googleusercontent.com" bind:value={clientId} />
        <button onclick={saveClientId} disabled={busy}>저장</button>
      </div>
      {#if !status.configured}
        <p class="hint">
          Google Cloud Console에서 '데스크톱 앱' OAuth 클라이언트 ID를 발급받아 위에 입력·저장하면
          아래 연결 버튼이 활성화됩니다. (로컬 기능은 ID 없이도 정상 동작합니다.)
        </p>
      {:else if status.connected}
        <p class="ok">● 연결됨</p>
        <div class="row">
          <button class="primary" onclick={syncAll} disabled={busy}>선택 메모 동기화</button>
          <button onclick={disconnect} disabled={busy}>연결 해제</button>
        </div>
      {:else}
        <p class="hint">아직 연결되지 않았습니다.</p>
        <button class="primary" onclick={connect} disabled={busy}>Google Calendar 연결</button>
      {/if}
    </div>
  {/if}

  {#if message}<pre class="msg">{message}</pre>{/if}
</section>

<style>
  .settings {
    flex: 1;
    overflow-y: auto;
    padding: 10px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  /* Card layout (responsive: cards/subcards wrap to a single column when narrow). */
  .cards-row {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
  }
  .cards-row > .card {
    flex: 1 1 280px;
  }
  .card {
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 12px;
  }
  .card-head {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    font-weight: 600;
    color: var(--fg);
    margin-bottom: 10px;
  }
  .card-head .ic {
    font-size: 15px;
  }
  .sub-head {
    font-size: 12px;
    margin-bottom: 8px;
  }
  .subcards {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
  }
  .subcards > .subcard {
    flex: 1 1 200px;
  }
  .subcard {
    background: color-mix(in srgb, var(--fg) 4%, transparent);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 10px;
  }
  .col {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  h3 {
    margin: 0;
    color: var(--accent);
    font-size: 14px;
  }
  h4 {
    margin: 0 0 6px;
    font-size: 12px;
    color: var(--fg);
  }
  .row {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
    margin-bottom: 6px;
  }
  .grow {
    flex: 1;
    min-width: 0;
  }
  .swatch {
    width: 46px;
    height: 28px;
    flex: 0 0 auto;
    padding: 2px;
    cursor: pointer;
  }
  .chk {
    display: flex;
    align-items: center;
    gap: 4px;
    color: var(--muted);
    font-size: 12px;
  }
  .chk input {
    width: auto;
  }
  .hint {
    color: var(--muted);
    font-size: 12px;
    margin: 4px 0;
  }
  h4.sub {
    margin-top: 10px;
  }
  .opacity {
    align-items: center;
  }
  .opacity input[type="range"] {
    flex: 1;
    width: auto;
    padding: 0;
    accent-color: var(--accent);
  }
  .opacity-val {
    font-size: 12px;
    color: var(--muted);
    min-width: 34px;
    text-align: right;
  }
  .ok {
    color: #8be29a;
    margin: 4px 0;
  }
  .msg {
    margin-top: 12px;
    padding: 8px;
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    font-size: 12px;
    white-space: pre-wrap;
    word-break: break-word;
    user-select: text;
  }
</style>
