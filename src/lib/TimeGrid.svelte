<script lang="ts">
  import { listEventsByRange } from "./api";
  import type { CalendarEvent } from "./types";
  import { bus } from "./store.svelte";
  import { addDays, eventStartDate, timeLabel, pad2 } from "./date";
  import { t, wd } from "./i18n";

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

  // Hour slot currently under the cursor, for the hover indicator.
  let hover = $state<{ day: string; hour: number } | null>(null);

  function hourAt(e: MouseEvent): number {
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const hour = Math.floor((e.clientY - rect.top) / HOUR);
    return Math.min(Math.max(hour, 0), 23);
  }

  function slotClick(d: string, e: MouseEvent) {
    onnew(d, hourAt(e));
  }

  function slotMove(d: string, e: MouseEvent) {
    hover = { day: d, hour: hourAt(e) };
  }

  function colLabel(d: string): string {
    const dt = new Date(d + "T00:00:00");
    return `${wd(dt.getDay())} ${dt.getMonth() + 1}/${dt.getDate()}`;
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
    <div class="gutter mini">{t("allDayBadge")}</div>
    {#each dates as d (d)}
      <div
        class="allday-col"
        role="button"
        tabindex="-1"
        title={t("addAllDay")}
        onclick={() => onnew(d)}
      >
        {#each allDayFor(d) as ev (ev.event_id)}
          <button
            class="chip"
            style:--chip={ev.color ?? "var(--accent)"}
            onclick={(e) => {
              e.stopPropagation();
              onedit(ev);
            }}
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
        onmousemove={(e) => slotMove(d, e)}
        onmouseleave={() => (hover = null)}
      >
        {#each hours as h (h)}
          <div class="line" style:top={`${h * HOUR}px`}></div>
        {/each}
        {#if hover && hover.day === d}
          <div class="hover" style:top={`${hover.hour * HOUR}px`} style:height={`${HOUR}px`}></div>
        {/if}
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
    /* Reserve the same scrollbar gutter as .body so columns stay aligned. */
    overflow-y: hidden;
    scrollbar-gutter: stable;
  }
  /* Give the all-day strip the height of one hour row. */
  .allday {
    min-height: 40px;
  }
  .allday-col {
    overflow-y: auto;
    cursor: pointer;
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
    /* Fill the all-day cell so a single chip matches the 1-hour row height. */
    flex: 1 1 auto;
    min-height: 0;
    display: flex;
    align-items: center;
  }
  .body {
    flex: 1;
    display: flex;
    overflow-y: auto;
    scrollbar-gutter: stable;
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
    /* Never let the flex column compress the rows below 40px, or the hour
       labels drift out of step with the 40px-per-hour grid lines and blocks. */
    flex-shrink: 0;
  }
  .col {
    flex: 1;
    position: relative;
    border-left: 1px solid var(--border);
    min-width: 0;
    cursor: pointer;
  }
  .line {
    position: absolute;
    left: 0;
    right: 0;
    border-top: 1px solid var(--border);
    opacity: 0.4;
  }
  .hover {
    position: absolute;
    left: 0;
    right: 0;
    background: color-mix(in srgb, var(--accent) 16%, transparent);
    border: 1px dashed var(--accent);
    box-sizing: border-box;
    pointer-events: none;
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
    font-size: 13px;
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
