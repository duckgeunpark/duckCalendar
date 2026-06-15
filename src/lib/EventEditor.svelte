<script lang="ts">
  import { createEvent, updateEvent, deleteEvent, listCalendars } from "./api";
  import type { CalendarEvent, Calendar } from "./types";
  import { bumpMemos } from "./store.svelte";
  import { addDays, eventStartDate, timeLabel, toRfc3339, pad2 } from "./date";
  import { t } from "./i18n";

  interface Props {
    date: string; // 'YYYY-MM-DD' of the day being viewed
    event: CalendarEvent | null; // null = compose a new event
    onclose: (changed: boolean) => void;
    /** Prefill when creating from a time-grid slot. */
    initial?: { day?: string; hour?: number };
  }
  let { date, event, onclose, initial }: Props = $props();

  let calendars = $state<Calendar[]>([]);

  let title = $state(event?.title ?? "");
  let description = $state(event?.description ?? "");
  let location = $state(event?.location ?? "");
  // 위치 입력칸 표시 여부. 기본은 끔(위치 지정 안 함); 기존 위치가 있으면 켜둔다.
  let showLocation = $state(!!event?.location);
  // Creating from a time slot starts as a timed event; otherwise default all-day.
  let allDay = $state(event ? event.all_day : initial?.hour === undefined);
  let day = $state(event ? eventStartDate(event.start_at) : (initial?.day ?? date));
  let startTime = $state(
    event && !event.all_day
      ? timeLabel(event.start_at)
      : initial?.hour !== undefined
        ? `${pad2(initial.hour)}:00`
        : "09:00",
  );
  let endTime = $state(
    event && !event.all_day
      ? timeLabel(event.end_at)
      : initial?.hour !== undefined
        ? `${pad2(Math.min(initial.hour + 1, 23))}:00`
        : "10:00",
  );
  let calendarId = $state(event?.calendar_id ?? "");

  let busy = $state(false);
  let error = $state("");

  $effect(() => {
    listCalendars()
      .then((c) => {
        calendars = c;
        if (!calendarId) {
          calendarId = c.find((x) => x.is_primary)?.calendar_id ?? c[0]?.calendar_id ?? "";
        }
      })
      .catch(() => (calendars = []));
  });

  function buildRange(): { start_at: string; end_at: string } | null {
    if (allDay) {
      return { start_at: day, end_at: addDays(day, 1) };
    }
    const start_at = toRfc3339(day, startTime);
    const end_at = toRfc3339(day, endTime);
    if (end_at <= start_at) {
      error = t("errEndAfterStart");
      return null;
    }
    return { start_at, end_at };
  }

  async function save() {
    if (!title.trim()) {
      error = t("errTitleRequired");
      return;
    }
    const range = buildRange();
    if (!range) return;
    busy = true;
    error = "";
    try {
      if (event) {
        await updateEvent({
          ...event,
          calendar_id: calendarId || event.calendar_id,
          title: title.trim(),
          description,
          location: showLocation ? location : "",
          all_day: allDay,
          ...range,
        });
      } else {
        await createEvent({
          calendar_id: calendarId || undefined,
          title: title.trim(),
          description,
          location: showLocation ? location : "",
          all_day: allDay,
          sync_enabled: false,
          ...range,
        });
      }
      bumpMemos();
      onclose(true);
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function remove() {
    if (!event) return;
    busy = true;
    error = "";
    try {
      await deleteEvent(event.event_id);
      bumpMemos();
      onclose(true);
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<section class="editor">
  <div class="head">
    <div class="head-left">
      <button class="ghost back" onclick={() => onclose(false)} title={t("backListTitle")}>{t("backList")}</button>
      <strong>{event ? t("editEvent") : t("newEvent")}</strong>
    </div>
    <input class="head-date" type="date" bind:value={day} />
    <div class="head-actions">
      <button class="primary" onclick={save} disabled={busy}>{t("save")}</button>
      <button class="ghost" onclick={() => onclose(false)} disabled={busy}>{t("cancel")}</button>
      {#if event}
        <button class="ghost danger" onclick={remove} disabled={busy} title={t("delete")}>✕</button>
      {/if}
    </div>
  </div>

  <div class="body">
    <input class="title-in" placeholder={t("titlePh")} bind:value={title} />

    <div class="row">
      <label class="chk">
        <input type="checkbox" bind:checked={allDay} /> {t("allDay")}
      </label>
      <label class="chk">
        <input type="checkbox" bind:checked={showLocation} /> {t("location")}
      </label>
    </div>

    {#if !allDay}
      <div class="row">
        <input type="time" bind:value={startTime} />
        <span class="dash">~</span>
        <input type="time" bind:value={endTime} />
      </div>
    {/if}

    {#if calendars.length > 1}
      <select bind:value={calendarId}>
        {#each calendars as c (c.calendar_id)}
          <option value={c.calendar_id}>{c.name}</option>
        {/each}
      </select>
    {/if}

    {#if showLocation}
      <input placeholder={t("locationPh")} bind:value={location} />
    {/if}
    <textarea placeholder={t("descPh")} bind:value={description}></textarea>

    {#if error}<p class="err">{error}</p>{/if}
  </div>
</section>

<style>
  .editor {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    padding: 8px 10px;
    gap: 6px;
    overflow: hidden;
  }
  /* Pinned single-row header: 목록·제목(좌) · 날짜(중앙) · 저장/취소/삭제(우). */
  .head {
    flex-shrink: 0;
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    gap: 8px;
  }
  .head-left {
    justify-self: start;
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }
  .head-date {
    justify-self: center;
    width: auto;
  }
  .head-actions {
    justify-self: end;
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .head-actions button {
    padding: 4px 8px;
  }
  /* Scrollable field area; grows to fill the space freed by moving the buttons up. */
  .body {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .head strong {
    color: var(--accent);
    font-size: 13px;
    white-space: nowrap;
  }
  .back {
    font-size: 12px;
    color: var(--muted);
    white-space: nowrap;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
  }
  .row input[type="time"] {
    width: auto;
    flex: 0 0 auto;
  }
  .dash {
    color: var(--muted);
  }
  .title-in {
    font-weight: 600;
    font-size: 15px;
  }
  textarea {
    min-height: 80px;
    flex: 1;
  }
  .chk {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--muted);
    font-size: 12px;
  }
  .chk input {
    width: auto;
  }
  select {
    font: inherit;
    color: var(--fg);
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 6px 8px;
  }
  .danger {
    color: var(--danger);
  }
  .err {
    color: var(--danger);
    margin: 0;
    font-size: 12px;
    word-break: break-word;
  }
</style>
