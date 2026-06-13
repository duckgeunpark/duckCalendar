<script lang="ts">
  import { createEvent, updateEvent, deleteEvent, listCalendars } from "./api";
  import type { CalendarEvent, Calendar } from "./types";
  import { bumpMemos } from "./store.svelte";
  import { addDays, eventStartDate, timeLabel, toRfc3339, pad2 } from "./date";

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
      error = "종료 시각이 시작 시각보다 늦어야 합니다.";
      return null;
    }
    return { start_at, end_at };
  }

  async function save() {
    if (!title.trim()) {
      error = "제목을 입력하세요.";
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
          location,
          all_day: allDay,
          ...range,
        });
      } else {
        await createEvent({
          calendar_id: calendarId || undefined,
          title: title.trim(),
          description,
          location,
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
    <button class="ghost back" onclick={() => onclose(false)} title="목록으로">‹ 목록</button>
    <strong>{event ? "일정 편집" : "새 일정"}</strong>
  </div>

  <input placeholder="제목" bind:value={title} />

  <label class="chk">
    <input type="checkbox" bind:checked={allDay} /> 종일
  </label>

  <div class="row">
    <input type="date" bind:value={day} />
    {#if !allDay}
      <input type="time" bind:value={startTime} />
      <span class="dash">~</span>
      <input type="time" bind:value={endTime} />
    {/if}
  </div>

  {#if calendars.length > 1}
    <select bind:value={calendarId}>
      {#each calendars as c (c.calendar_id)}
        <option value={c.calendar_id}>{c.name}</option>
      {/each}
    </select>
  {/if}

  <input placeholder="위치 (선택)" bind:value={location} />
  <textarea placeholder="설명 (선택)" bind:value={description}></textarea>

  {#if error}<p class="err">{error}</p>{/if}

  <div class="actions">
    <button class="primary" onclick={save} disabled={busy}>저장</button>
    <button class="ghost" onclick={() => onclose(false)} disabled={busy}>취소</button>
    {#if event}
      <button class="ghost danger" onclick={remove} disabled={busy} title="삭제">🗑 삭제</button>
    {/if}
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
    overflow-y: auto;
  }
  .head {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .head strong {
    color: var(--accent);
    font-size: 13px;
  }
  .back {
    font-size: 12px;
    color: var(--muted);
  }
  .row {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
  }
  .row input[type="date"],
  .row input[type="time"] {
    width: auto;
    flex: 0 0 auto;
  }
  .dash {
    color: var(--muted);
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
  .actions {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }
  .danger {
    color: #f0a0a0;
  }
  .err {
    color: #f0a0a0;
    margin: 0;
    font-size: 12px;
    word-break: break-word;
  }
</style>
