<script lang="ts">
  import { listEventsByRange } from "./api";
  import type { CalendarEvent } from "./types";
  import {
    daysInMonth,
    firstWeekday,
    fmt,
    todayParts,
    WEEKDAYS,
    MONTH_LABELS,
    monthRange,
    addDays,
    eventStartDate,
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

  let dayEvents = $state<Map<string, CalendarEvent[]>>(new Map());
  let showPicker = $state(false);
  // Year shown inside the open month picker (independent of viewYear until chosen).
  let pickerYear = $state(viewYear);

  // Re-fetch this month's events whenever the month or data changes. A multi-day
  // event is placed on every date in its [start, end) span.
  $effect(() => {
    const y = viewYear;
    const m = viewMonth;
    void bus.memoVersion;
    const { start, end } = monthRange(y, m);
    listEventsByRange(start, end)
      .then((list) => {
        const map = new Map<string, CalendarEvent[]>();
        for (const ev of list) {
          const last = ev.all_day ? addDays(ev.end_at, -1) : eventStartDate(ev.end_at);
          let d = eventStartDate(ev.start_at);
          // Guard against pathological ranges; cap at 60 iterations.
          for (let i = 0; i < 60; i++) {
            const arr = map.get(d) ?? [];
            arr.push(ev);
            map.set(d, arr);
            if (d >= last) break;
            d = addDays(d, 1);
          }
        }
        dayEvents = map;
      })
      .catch((e) => console.error("listEventsByRange failed", e));
  });

  // Build only the weeks needed for the displayed month, filling leading/trailing
  // slots with adjacent months' days (shown greyed out).
  type Cell = { date: string; day: number; inMonth: boolean };
  let cells = $derived.by(() => {
    const lead = firstWeekday(viewYear, viewMonth);
    const total = daysInMonth(viewYear, viewMonth);
    const pm = viewMonth === 1 ? 12 : viewMonth - 1;
    const py = viewMonth === 1 ? viewYear - 1 : viewYear;
    const nm = viewMonth === 12 ? 1 : viewMonth + 1;
    const ny = viewMonth === 12 ? viewYear + 1 : viewYear;
    const prevTotal = daysInMonth(py, pm);
    const out: Cell[] = [];
    for (let i = lead - 1; i >= 0; i--) {
      const d = prevTotal - i;
      out.push({ date: fmt(py, pm, d), day: d, inMonth: false });
    }
    for (let d = 1; d <= total; d++) {
      out.push({ date: fmt(viewYear, viewMonth, d), day: d, inMonth: true });
    }
    const targetLength = Math.ceil(out.length / 7) * 7;
    let nd = 1;
    while (out.length < targetLength) {
      out.push({ date: fmt(ny, nm, nd), day: nd, inMonth: false });
      nd++;
    }
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
  function openPicker() {
    pickerYear = viewYear;
    showPicker = !showPicker;
  }
  function pickMonth(m: number) {
    viewYear = pickerYear;
    viewMonth = m;
    showPicker = false;
  }
  function stopPickerEvent(event: Event) {
    event.stopPropagation();
  }
  function closePicker(event: MouseEvent) {
    event.stopPropagation();
    showPicker = false;
  }
  function pickerToday(event: MouseEvent) {
    event.preventDefault();
    event.stopPropagation();
    showPicker = false;
    viewYear = today.y;
    viewMonth = today.m;
    onselect(todayStr);
  }
</script>

<section class="calendar">
  <div class="nav">
    <button class="ghost arrow" onclick={prevMonth} aria-label="이전 달">‹</button>
    <button class="ghost label" onclick={openPicker} title="월 선택">
      {viewYear}년 {MONTH_LABELS[viewMonth - 1]}
    </button>
    <button class="ghost arrow" onclick={nextMonth} aria-label="다음 달">›</button>
  </div>

  {#if showPicker}
    <div class="picker-backdrop" role="presentation" onclick={closePicker}></div>
    <div class="picker" role="dialog" tabindex="-1" onpointerdown={stopPickerEvent}>
      <div class="picker-year">
        <button class="ghost" onclick={() => (pickerYear -= 1)} aria-label="이전 해">‹</button>
        <span>{pickerYear}년</span>
        <button class="ghost" onclick={() => (pickerYear += 1)} aria-label="다음 해">›</button>
      </div>
      <div class="picker-grid">
        {#each MONTH_LABELS as ml, i}
          <button
            class="ghost"
            class:cur={pickerYear === viewYear && i + 1 === viewMonth}
            onclick={() => pickMonth(i + 1)}>{ml}</button>
        {/each}
      </div>
      <button class="ghost picker-today" onpointerdown={stopPickerEvent} onclick={pickerToday}>
        오늘로 이동
      </button>
    </div>
  {/if}

  <div class="grid weekdays">
    {#each WEEKDAYS as w, i}
      <div class="wd" class:sun={i === 0} class:sat={i === 6}>{w}</div>
    {/each}
  </div>

  <div class="grid days">
    {#each cells as cell, i}
      {@const items = cell.inMonth ? (dayEvents.get(cell.date) ?? []) : []}
      <button
        class="cell"
        class:out={!cell.inMonth}
        class:today={cell.date === todayStr}
        class:selected={cell.date === selectedDate}
        onclick={() => onselect(cell.date)}
        title="클릭하여 그날 보기"
      >
        <span class="num" class:sun={i % 7 === 0} class:sat={i % 7 === 6}>{cell.day}</span>
        {#if items.length}
          <span class="memos">
            {#each items.slice(0, 3) as ev (ev.event_id)}
              <span class="m" class:ev={!ev.all_day} style:--chip={ev.color ?? "var(--accent)"}>
                {ev.title}
              </span>
            {/each}
            {#if items.length > 3}<span class="more">+{items.length - 3}</span>{/if}
          </span>
        {/if}
      </button>
    {/each}
  </div>
</section>

<style>
  .calendar {
    padding: 8px 10px;
    position: relative;
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
  .nav {
    display: grid;
    grid-template-columns: 32px 1fr 32px;
    align-items: center;
    margin-bottom: 6px;
  }
  .arrow {
    font-size: 16px;
  }
  .label {
    font-weight: 600;
    text-align: center;
    font-size: 14px;
  }
  .picker-backdrop {
    position: absolute;
    inset: 0;
    z-index: 19;
  }
  .picker {
    position: absolute;
    z-index: 20;
    left: 50%;
    transform: translateX(-50%);
    top: 34px;
    width: 236px;
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 8px;
    box-shadow: 0 6px 20px rgba(0, 0, 0, 0.4);
  }
  .picker-year {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 6px;
    font-weight: 600;
  }
  .picker-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 4px;
  }
  .picker-grid button {
    padding: 6px 0;
    font-size: 12px;
  }
  .picker-grid button.cur {
    background: var(--accent);
    color: var(--accent-fg);
    border-color: var(--accent);
    font-weight: 700;
  }
  .picker-today {
    width: 100%;
    margin-top: 6px;
    font-size: 12px;
    color: var(--muted);
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: 2px;
  }
  .days {
    flex: 1;
    grid-auto-rows: 1fr;
    min-height: 0;
  }
  .wd {
    text-align: center;
    color: var(--muted);
    font-size: 11px;
    padding: 2px 0;
  }
  .cell {
    position: relative;
    min-height: 0;
    display: flex;
    flex-direction: column;
    align-items: stretch;
    justify-content: flex-start;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 2px 3px;
    overflow: hidden;
    text-align: left;
  }
  .cell.empty {
    border: none;
    background: transparent;
  }
  .cell:hover {
    background: rgba(255, 255, 255, 0.09);
  }
  .cell.today {
    border-color: var(--accent);
  }
  .cell.selected {
    background: var(--panel-2);
    border-color: var(--accent);
  }
  .num {
    font-size: 12px;
    font-weight: 600;
    line-height: 1.3;
  }
  .cell.today .num {
    color: var(--accent);
  }
  .cell.out {
    background: transparent;
    border-color: transparent;
  }
  .cell.out .num,
  .cell.out .num.sun,
  .cell.out .num.sat {
    color: var(--muted);
    opacity: 0.45;
    font-weight: 400;
  }
  .cell.out:hover {
    background: rgba(255, 255, 255, 0.05);
  }
  .sun {
    color: #ff8a80;
  }
  .sat {
    color: #82b1ff;
  }
  .memos {
    display: flex;
    flex-direction: column;
    gap: 1px;
    margin-top: 1px;
    min-height: 0;
  }
  .m {
    font-size: 9px;
    line-height: 1.35;
    padding: 0 3px;
    border-radius: 3px;
    background: var(--panel);
    color: var(--fg);
    border-left: 3px solid var(--chip, var(--accent));
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .m.ev {
    background: #3a3d44;
  }
  .more {
    font-size: 9px;
    color: var(--muted);
    padding: 0 3px;
  }
</style>
