<script lang="ts">
  import { listEventsByRange } from "./api";
  import type { CalendarEvent } from "./types";
  import { bus } from "./store.svelte";
  import { addDays, timeLabel } from "./date";
  import { t, wd } from "./i18n";

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

  // Same compact label as the week/day time-grid headers: "TUE 6/16".
  let dayLabel = $derived.by(() => {
    const dt = new Date(date + "T00:00:00");
    return `${wd(dt.getDay())} ${dt.getMonth() + 1}/${dt.getDate()}`;
  });

  function timeText(ev: CalendarEvent): string {
    if (ev.all_day) return t("allDayBadge");
    const s = timeLabel(ev.start_at);
    const e = timeLabel(ev.end_at);
    return e ? `${s}–${e}` : s;
  }
</script>

<section class="day">
  <div class="day-head">
    <strong>{dayLabel}</strong>
    <button class="ghost back" onclick={onback} title={t("monthViewTitle")}>{t("monthViewBtn")}</button>
  </div>

  <button class="primary add" onclick={onnew}>{t("addEvent")}</button>

  {#if events.length === 0}
    <p class="muted">{t("noEvents")}</p>
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
    font-size: 15px;
  }
  .item .c {
    font-size: 12px;
    color: var(--muted);
    white-space: pre-wrap;
    word-break: break-word;
  }
</style>
