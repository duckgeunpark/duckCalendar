<script lang="ts">
  import {
    googleStatus,
    googleConnect,
    googleDisconnect,
    syncSelected,
    exportToFile,
  } from "./api";
  import type { ExportFormat, ExportScope, GoogleStatus } from "./types";

  interface Props {
    selectedDate: string; // 'YYYY-MM-DD'
    onclose: () => void;
  }
  let { selectedDate, onclose }: Props = $props();

  let status = $state<GoogleStatus>({ configured: false, connected: false });
  let busy = $state(false);
  let message = $state("");

  let scope = $state<ExportScope>("month");

  $effect(() => {
    refreshStatus();
  });

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
    <h3>설정</h3>
    <button class="ghost" onclick={onclose}>닫기 ✕</button>
  </div>

  <div class="block">
    <h4>데이터 추출</h4>
    <div class="row">
      <label class="chk"><input type="radio" name="scope" value="date" bind:group={scope} /> 이 날짜</label>
      <label class="chk"><input type="radio" name="scope" value="month" bind:group={scope} /> 이 달</label>
      <label class="chk"><input type="radio" name="scope" value="all" bind:group={scope} /> 전체</label>
    </div>
    <div class="row">
      <button onclick={() => doExport("json")} disabled={busy}>JSON</button>
      <button onclick={() => doExport("csv")} disabled={busy}>CSV</button>
      <button onclick={() => doExport("ics")} disabled={busy}>ICS</button>
    </div>
    <p class="hint">ICS에는 '일정으로 지정'된 메모만 포함됩니다.</p>
  </div>

  <div class="block">
    <h4>Google Calendar 연동</h4>
    {#if !status.configured}
      <p class="hint">
        GOOGLE_CLIENT_ID가 설정되지 않아 연동을 사용할 수 없습니다. README의 연동 설정을 참고하세요.
        (로컬 기능은 정상 동작합니다.)
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

  {#if message}<pre class="msg">{message}</pre>{/if}
</section>

<style>
  .settings {
    flex: 1;
    overflow-y: auto;
    padding: 8px 10px;
  }
  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
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
  .block {
    margin-top: 12px;
    padding-top: 10px;
    border-top: 1px solid var(--border);
  }
  .row {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
    margin-bottom: 6px;
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
