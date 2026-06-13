<script lang="ts">
  import { listEventsByRange } from "./api";
  import type { CalendarEvent } from "./types";
  import { bus } from "./store.svelte";
  import { WEEKDAYS, addDays, eventStartDate, timeLabel, pad2 } from "./date";

  interface Props {
    dates: string[]; // column dates 'YYYY-MM-DD' (1 = day view, 7 = week view)
    onedit: (ev: CalendarEvent) => void;
    onnew: (day: string, hour: number) => void;
  }
  let { dates, onedit, onnew }: Props = $props();

  const HOUR = 40; // px per hour row
  const hours = Array.from({ length: 24 }, (_, i) => i);

  let events = $state<CalendarEvent[]>([]);

  $effect(() => {
    const ds = dates;
    void bus.memoVersion;
    if (ds.length === 0) return;
    listEventsByRange(ds[0], addDays(ds[ds.length - 1], 1))
      .then((m) => (events = m))
      .catch((e) => console.error("listEventsByRange failed", e));
  });

  const allDay = $derived(events.filter((e) => e.all_day));
  const timed = $derived(events.filter((e) => !e.all_day));

  function dayStartMs(d: string): number {
    return new Date(d + "T00:00:00").getTime();
  }

  // Timed event blocks that intersect a given column date.
  function blocksFor(d: string) {
    const ds = dayStartMs(d);
    const de = ds + 86_400_000;
    const out: { ev: CalendarEvent; top: number; height: number }[] = [];
    for (const ev of timed) {
      const s = new Date(ev.start_at).getTime();
      const e = new Date(ev.end_at).getTime();
      if (Number.isNaN(s) || Number.isNaN(e) || e <= ds || s >= de) continue;
      const top = ((Math.max(s, ds) - ds) / 3_600_000) * HOUR;
      const height = Math.max(((Math.min(e, de) - Math.max(s, ds)) / 3_600_000) * HOUR, 16);
      out.push({ ev, top, height });
    }
    return out;
  }

  // All-day events covering a given column date.
  function allDayFor(d: string) {
    return allDay.filter((e) => {
      const last = addDays(e.end_at, -1); // inclusive last day
      return d >= eventStartDate(e.start_at) && d <= last;
    });
  }

  function slotClick(d: string, e: MouseEvent) {
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const hour = Math.floor((e.clientY - rect.top) / HOUR);
    onnew(d, Math.min(Math.max(hour, 0), 23));
  }

  function colLabel(d: string): string {
    const dt = new Date(d + "T00:00:00");
    return `${WEEKDAYS[dt.getDay()]} ${dt.getDate()}`;
  }
</script>

<section class="grid">
  <!-- Header -->
  <div class="head">
    <div class="gutter"></div>
    {#each dates as d (d)}
      <div class="col-head">{colLabel(d)}</div>
    {/each}
  </div>

  <!-- All-day strip -->
  <div class="allday">
    <div class="gutter mini">종일</div>
    {#each dates as d (d)}
      <div class="allday-col">
        {#each allDayFor(d) as ev (ev.event_id)}
          <button
            class="chip"
            style:--chip={ev.color ?? "var(--accent)"}
            onclick={() => onedit(ev)}
            title={ev.title}>{ev.title}</button>
        {/each}
      </div>
    {/each}
  </div>

  <!-- Scrollable time body -->
  <div class="body">
    <div class="gutter hours">
      {#each hours as h (h)}
        <div class="hour" style:height={`${HOUR}px`}>{pad2(h)}:00</div>
      {/each}
    </div>
    {#each dates as d (d)}
      <div
        class="col"
        style:height={`${HOUR * 24}px`}
        role="button"
        tabindex="-1"
        onclick={(e) => slotClick(d, e)}
      >
        {#each hours as h (h)}
          <div class="line" style:top={`${h * HOUR}px`}></div>
        {/each}
        {#each blocksFor(d) as b (b.ev.event_id)}
          <button
            class="block"
            style:top={`${b.top}px`}
            style:height={`${b.height}px`}
            style:--chip={b.ev.color ?? "var(--accent)"}
            onclick={(e) => {
              e.stopPropagation();
              onedit(b.ev);
            }}
          >
            <span class="bt">{b.ev.title}</span>
            <span class="bw">{timeLabel(b.ev.start_at)}–{timeLabel(b.ev.end_at)}</span>
          </button>
        {/each}
      </div>
    {/each}
  </div>
</section>

<style>
  .grid {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    overflow: hidden;
  }
  .head,
  .allday {
    display: flex;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .gutter {
    width: 46px;
    flex-shrink: 0;
  }
  .gutter.mini {
    font-size: 10px;
    color: var(--muted);
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .col-head {
    flex: 1;
    text-align: center;
    font-size: 12px;
    padding: 4px 0;
    color: var(--fg);
    border-left: 1px solid var(--border);
  }
  .allday-col {
    flex: 1;
    min-width: 0;
    border-left: 1px solid var(--border);
    padding: 2px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .chip {
    font-size: 10px;
    text-align: left;
    padding: 1px 4px;
    border-radius: 3px;
    background: var(--panel);
    color: var(--fg);
    border: none;
    border-left: 3px solid var(--chip, var(--accent));
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .body {
    flex: 1;
    display: flex;
    overflow-y: auto;
    min-height: 0;
  }
  .gutter.hours {
    display: flex;
    flex-direction: column;
  }
  .hour {
    font-size: 10px;
    color: var(--muted);
    text-align: right;
    padding-right: 4px;
    box-sizing: border-box;
  }
  .col {
    flex: 1;
    position: relative;
    border-left: 1px solid var(--border);
    min-width: 0;
  }
  .line {
    position: absolute;
    left: 0;
    right: 0;
    border-top: 1px solid var(--border);
    opacity: 0.4;
  }
  .block {
    position: absolute;
    left: 2px;
    right: 2px;
    overflow: hidden;
    text-align: left;
    background: color-mix(in srgb, var(--chip, var(--accent)) 28%, var(--panel));
    border: none;
    border-left: 3px solid var(--chip, var(--accent));
    border-radius: 3px;
    padding: 1px 4px;
    display: flex;
    flex-direction: column;
    color: var(--fg);
  }
  .bt {
    font-size: 11px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .bw {
    font-size: 9px;
    color: var(--muted);
  }
</style>
