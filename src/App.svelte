<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import Calendar from "./lib/Calendar.svelte";
  import DayView from "./lib/DayView.svelte";
  import TimeGrid from "./lib/TimeGrid.svelte";
  import EventEditor from "./lib/EventEditor.svelte";
  import Settings from "./lib/Settings.svelte";
  import { todayParts, fmt, weekDates } from "./lib/date";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { getSetting, setExpanded } from "./lib/api";
  import { ui, bumpMemos } from "./lib/store.svelte";
  import type { CalendarEvent } from "./lib/types";

  const t = todayParts();
  let selectedDate = $state(fmt(t.y, t.m, t.d));
  let view = $state<"month" | "week" | "day">("month");
  let expanded = $state(false);
  let showSettings = $state(false);

  // Editor overlay shared by all views. undefined = closed, null = new event,
  // CalendarEvent = editing. `editorInitial` prefills a new event's time.
  let editing = $state<CalendarEvent | null | undefined>(undefined);
  let editorInitial = $state<{ day?: string; hour?: number } | undefined>(undefined);

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

    const unMemo = listen("memos-changed", () => bumpMemos());
    const unSettings = listen("open-settings", () => (showSettings = true));
    return () => {
      unMemo.then((f) => f());
      unSettings.then((f) => f());
    };
  });

  // Reflect opacity + font scale on this window's root.
  $effect(() => {
    document.documentElement.style.setProperty("--app-alpha", String(ui.opacity));
  });
  $effect(() => {
    document.documentElement.style.setProperty("zoom", String(ui.scale));
  });

  let weekCols = $derived(weekDates(selectedDate));

  function selectDay(date: string) {
    selectedDate = date;
    view = "day";
  }
  function backToMonth(e: MouseEvent) {
    e.preventDefault();
    if (editing !== undefined) closeEditor();
    else view = "month";
  }

  async function toggleExpanded() {
    expanded = !expanded;
    if (!expanded && view === "week") view = "month";
    try {
      await setExpanded(expanded);
    } catch (e) {
      console.error("setExpanded failed", e);
    }
  }

  function openNew(day?: string, hour?: number) {
    if (day) selectedDate = day;
    editorInitial = hour !== undefined ? { day, hour } : undefined;
    editing = null;
  }
  function openEdit(ev: CalendarEvent) {
    editing = ev;
  }
  function closeEditor() {
    editing = undefined;
    editorInitial = undefined;
  }
</script>

<header class="titlebar" class:widget={ui.windowMode === "desktop"} data-tauri-drag-region>
  <span class="title" data-tauri-drag-region>duckCalendar</span>
  <div class="win-actions">
    <button class="ghost btn-expand" title={expanded ? "축소" : "확장"} onclick={toggleExpanded}>
      {expanded ? "⊟" : "⊞"}
    </button>
    <button class="ghost btn-settings" title="설정" onclick={() => (showSettings = !showSettings)}>⚙</button>
    <button class="ghost btn-min" title="숨기기" onclick={() => appWindow.hide()}>—</button>
    <button class="ghost btn-close" title="닫기" onclick={() => appWindow.close()}>✕</button>
  </div>
</header>

{#if expanded && !showSettings}
  <nav class="viewbar">
    <button class="ghost" class:active={view === "month"} onclick={() => (view = "month")}>월</button>
    <button class="ghost" class:active={view === "week"} onclick={() => (view = "week")}>주</button>
    <button class="ghost" class:active={view === "day"} onclick={() => (view = "day")}>일</button>
    <span class="sel">{selectedDate}</span>
  </nav>
{/if}

<main oncontextmenu={backToMonth}>
  {#if showSettings}
    <Settings selectedDate={selectedDate} onclose={() => (showSettings = false)} />
  {:else if editing !== undefined}
    <EventEditor date={selectedDate} event={editing} initial={editorInitial} onclose={closeEditor} />
  {:else if view === "week"}
    <TimeGrid dates={weekCols} onedit={openEdit} onnew={openNew} />
  {:else if view === "day" && expanded}
    <TimeGrid dates={[selectedDate]} onedit={openEdit} onnew={openNew} />
  {:else if view === "day"}
    <DayView
      date={selectedDate}
      onback={() => (view = "month")}
      onedit={openEdit}
      onnew={() => openNew()}
    />
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
  /* Desktop-pinned widget: hide the name + window chrome. */
  .titlebar.widget .title,
  .titlebar.widget .btn-expand,
  .titlebar.widget .btn-settings,
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
  .viewbar {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .viewbar .active {
    background: var(--accent);
    color: var(--accent-fg);
    border-color: var(--accent);
  }
  .viewbar .sel {
    margin-left: auto;
    font-size: 12px;
    color: var(--muted);
  }
  main {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
</style>
