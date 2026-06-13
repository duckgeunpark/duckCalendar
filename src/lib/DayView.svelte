<script lang="ts">
  import { listEventsByRange } from "./api";
  import type { CalendarEvent } from "./types";
  import { bus } from "./store.svelte";
  import { WEEKDAYS, addDays, timeLabel } from "./date";

  interface Props {
    date: string; // 'YYYY-MM-DD'
    onback: () => void;
    onedit: (ev: CalendarEvent) => void;
    onnew: () => void;
  }
  let { date, onback, onedit, onnew }: Props = $props();

  let events = $state<CalendarEvent[]>([]);

  $effect(() => {
    void date;
    void bus.memoVersion;
    listEventsByRange(date, addDays(date, 1))
      .then((m) => (events = m))
      .catch((e) => console.error("listEventsByRange failed", e));
  });

  let weekday = $derived(WEEKDAYS[new Date(date + "T00:00:00").getDay()]);

  function timeText(ev: CalendarEvent): string {
    if (ev.all_day) return "종일";
    const s = timeLabel(ev.start_at);
    const e = timeLabel(ev.end_at);
    return e ? `${s}–${e}` : s;
  }
</script>

<section class="day">
  <div class="day-head">
    <strong>{date} ({weekday})</strong>
    <button class="ghost back" onclick={onback} title="월별 보기 (우클릭으로도 가능)">월별 ▦</button>
  </div>

  <button class="primary add" onclick={onnew}>＋ 새 일정</button>

  {#if events.length === 0}
    <p class="muted">일정이 없습니다. 위 버튼으로 추가하세요.</p>
  {:else}
    <ul class="list">
      {#each events as ev (ev.event_id)}
        <li>
          <button
            class="ghost item"
            style:--chip={ev.color ?? "var(--accent)"}
            onclick={() => onedit(ev)}
          >
            <span class="when">{timeText(ev)}</span>
            <span class="t">{ev.title}</span>
            {#if ev.location}<span class="c">📍 {ev.location}</span>{/if}
            {#if ev.description}<span class="c">{ev.description}</span>{/if}
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  .day {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    padding: 8px 10px;
    gap: 8px;
  }
  .day-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .day-head strong {
    color: var(--accent);
    font-size: 14px;
  }
  .back {
    font-size: 11px;
    color: var(--muted);
  }
  .add {
    width: 100%;
  }
  .muted {
    color: var(--muted);
    margin: 4px 0;
  }
  .list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
    overflow-y: auto;
  }
  .item {
    width: 100%;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 2px;
    text-align: left;
    background: var(--panel);
    border-color: var(--border);
    border-left: 4px solid var(--chip, var(--accent));
  }
  .item .when {
    font-size: 11px;
    color: var(--muted);
  }
  .item .t {
    font-weight: 600;
    font-size: 13px;
  }
  .item .c {
    font-size: 12px;
    color: var(--muted);
    white-space: pre-wrap;
    word-break: break-word;
  }
</style>
