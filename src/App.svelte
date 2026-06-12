<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import Calendar from "./lib/Calendar.svelte";
  import DayView from "./lib/DayView.svelte";
  import { todayParts, fmt } from "./lib/date";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { getSetting } from "./lib/api";
  import { ui, bumpMemos } from "./lib/store.svelte";

  const t = todayParts();
  let selectedDate = $state(fmt(t.y, t.m, t.d));
  let view = $state<"month" | "day">("month");

  const appWindow = getCurrentWindow();

  onMount(() => {
    (async () => {
      const mode = await getSetting("window_mode");
      if (mode === "top" || mode === "desktop" || mode === "normal") ui.windowMode = mode;
      const op = await getSetting("opacity");
      const n = op ? Number(op) : NaN;
      if (!Number.isNaN(n) && n >= 0.3 && n <= 1) ui.opacity = n;
      const sc = await getSetting("ui_scale");
      const s = sc ? Number(sc) : NaN;
      if (!Number.isNaN(s) && s >= 0.7 && s <= 1.5) ui.scale = s;
    })();

    // Memo edits happen in a separate window; refresh when notified.
    const unMemo = listen("memos-changed", () => bumpMemos());
    // Appearance changes come from the settings window.
    const unAppear = listen<{ windowMode: typeof ui.windowMode; opacity: number; scale: number }>(
      "appearance-changed",
      (e) => {
        ui.windowMode = e.payload.windowMode;
        ui.opacity = e.payload.opacity;
        ui.scale = e.payload.scale;
      },
    );
    return () => {
      unMemo.then((f) => f());
      unAppear.then((f) => f());
    };
  });

  // Reflect opacity + font scale on this window's root.
  $effect(() => {
    document.documentElement.style.setProperty("--app-alpha", String(ui.opacity));
  });
  $effect(() => {
    document.documentElement.style.setProperty("zoom", String(ui.scale));
  });

  function selectDay(date: string) {
    selectedDate = date;
    view = "day";
  }
  function backToMonth(e: MouseEvent) {
    // Right-click returns to the month calendar.
    e.preventDefault();
    view = "month";
  }
</script>

<header class="titlebar" class:widget={ui.windowMode === "desktop"} data-tauri-drag-region>
  <span class="title" data-tauri-drag-region>duckCalendar</span>
  <div class="win-actions">
    <button class="ghost btn-min" title="숨기기" onclick={() => appWindow.hide()}>—</button>
    <button class="ghost btn-close" title="닫기" onclick={() => appWindow.close()}>✕</button>
  </div>
</header>

<main oncontextmenu={backToMonth}>
  {#if view === "day"}
    <DayView date={selectedDate} onback={() => (view = "month")} />
  {:else}
    <Calendar {selectedDate} onselect={selectDay} />
  {/if}
</main>

<style>
  .titlebar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 28px;
    padding: 0 6px 0 10px;
    background: rgba(var(--bg-rgb), 0.6);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  /* Desktop-pinned widget: hide the name + minimize/close chrome. */
  .titlebar.widget .title,
  .titlebar.widget .btn-min,
  .titlebar.widget .btn-close {
    display: none;
  }
  .titlebar.widget {
    background: transparent;
    border-bottom: none;
    height: 12px;
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
