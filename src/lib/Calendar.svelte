<script lang="ts">
  import { listMemoDates } from "./api";
  import {
    daysInMonth,
    firstWeekday,
    fmt,
    todayParts,
    WEEKDAYS,
    MONTH_LABELS,
  } from "./date";
  import { bus } from "./store.svelte";

  interface Props {
    selectedDate: string;
    onselect: (date: string) => void;
  }
  let { selectedDate, onselect }: Props = $props();

  const today = todayParts();
  const todayStr = fmt(today.y, today.m, today.d);

  // Displayed month derives its initial value from the selected date.
  let viewYear = $state(Number(selectedDate.slice(0, 4)));
  let viewMonth = $state(Number(selectedDate.slice(5, 7)));

  let memoDates = $state<Set<string>>(new Set());

  // Re-fetch the highlighted (has-memo) dates whenever the month or memos change.
  $effect(() => {
    const y = viewYear;
    const m = viewMonth;
    // Touch memoVersion so this effect re-runs after memo edits.
    void bus.memoVersion;
    listMemoDates(y, m)
      .then((dates) => (memoDates = new Set(dates)))
      .catch((e) => console.error("listMemoDates failed", e));
  });

  // Build the cell grid: leading blanks + each day.
  let cells = $derived.by(() => {
    const lead = firstWeekday(viewYear, viewMonth);
    const total = daysInMonth(viewYear, viewMonth);
    const out: (number | null)[] = [];
    for (let i = 0; i < lead; i++) out.push(null);
    for (let d = 1; d <= total; d++) out.push(d);
    return out;
  });

  function prevMonth() {
    if (viewMonth === 1) {
      viewMonth = 12;
      viewYear -= 1;
    } else {
      viewMonth -= 1;
    }
  }
  function nextMonth() {
    if (viewMonth === 12) {
      viewMonth = 1;
      viewYear += 1;
    } else {
      viewMonth += 1;
    }
  }
  function goToday() {
    viewYear = today.y;
    viewMonth = today.m;
    onselect(todayStr);
  }
  function dateOf(day: number): string {
    return fmt(viewYear, viewMonth, day);
  }
</script>

<section class="calendar">
  <div class="nav">
    <button class="ghost" onclick={prevMonth} aria-label="이전 달">‹</button>
    <button class="ghost label" onclick={goToday} title="오늘로 이동">
      {viewYear}년 {MONTH_LABELS[viewMonth - 1]}
    </button>
    <button class="ghost" onclick={nextMonth} aria-label="다음 달">›</button>
  </div>

  <div class="grid weekdays">
    {#each WEEKDAYS as w, i}
      <div class="wd" class:sun={i === 0} class:sat={i === 6}>{w}</div>
    {/each}
  </div>

  <div class="grid days">
    {#each cells as day, i}
      {#if day === null}
        <div class="cell empty"></div>
      {:else}
        {@const ds = dateOf(day)}
        <button
          class="cell"
          class:today={ds === todayStr}
          class:selected={ds === selectedDate}
          class:sun={i % 7 === 0}
          class:sat={i % 7 === 6}
          onclick={() => onselect(ds)}
        >
          <span>{day}</span>
          {#if memoDates.has(ds)}<i class="dot"></i>{/if}
        </button>
      {/if}
    {/each}
  </div>
</section>

<style>
  .calendar {
    padding: 8px 10px;
  }
  .nav {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 6px;
  }
  .label {
    font-weight: 600;
    flex: 1;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: 2px;
  }
  .wd {
    text-align: center;
    color: var(--muted);
    font-size: 11px;
    padding: 2px 0;
  }
  .cell {
    position: relative;
    aspect-ratio: 1 / 1;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: 1px solid transparent;
    border-radius: 6px;
    padding: 0;
  }
  .cell.empty {
    border: none;
  }
  .cell:hover {
    background: var(--panel-2);
  }
  .cell.today {
    border-color: var(--accent);
  }
  .cell.selected {
    background: var(--accent);
    color: var(--accent-fg);
    font-weight: 700;
  }
  .sun {
    color: #ff8a80;
  }
  .sat {
    color: #82b1ff;
  }
  .cell.selected.sun,
  .cell.selected.sat {
    color: var(--accent-fg);
  }
  .dot {
    position: absolute;
    bottom: 4px;
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: var(--accent);
  }
  .cell.selected .dot {
    background: var(--accent-fg);
  }
</style>
