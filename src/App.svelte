<script lang="ts">
  import Calendar from "./lib/Calendar.svelte";
  import MemoPanel from "./lib/MemoPanel.svelte";
  import Settings from "./lib/Settings.svelte";
  import { todayParts, fmt } from "./lib/date";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  const t = todayParts();
  let selectedDate = $state(fmt(t.y, t.m, t.d));
  let showSettings = $state(false);

  const appWindow = getCurrentWindow();
</script>

<header class="titlebar" data-tauri-drag-region>
  <span class="title" data-tauri-drag-region>🦆 duckCalendar</span>
  <div class="win-actions">
    <button class="ghost" title="설정" onclick={() => (showSettings = !showSettings)}>⚙</button>
    <button class="ghost" title="숨기기" onclick={() => appWindow.hide()}>—</button>
    <button class="ghost" title="닫기" onclick={() => appWindow.close()}>✕</button>
  </div>
</header>

<main>
  {#if showSettings}
    <Settings {selectedDate} onclose={() => (showSettings = false)} />
  {:else}
    <Calendar {selectedDate} onselect={(d) => (selectedDate = d)} />
    <MemoPanel {selectedDate} />
  {/if}
</main>

<style>
  .titlebar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 30px;
    padding: 0 6px 0 10px;
    background: var(--panel);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .title {
    font-weight: 600;
    font-size: 12px;
    pointer-events: none;
  }
  .win-actions {
    display: flex;
    gap: 2px;
  }
  .win-actions button {
    padding: 2px 7px;
    line-height: 1;
  }
  main {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
</style>
