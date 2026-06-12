<script lang="ts">
  import {
    listMemosByDate,
    createMemo,
    updateMemo,
    deleteMemo,
    googleStatus,
    syncMemo,
    syncStatusMap,
  } from "./api";
  import type { Memo } from "./types";
  import { bumpMemos } from "./store.svelte";

  interface Props {
    selectedDate: string;
  }
  let { selectedDate }: Props = $props();

  let memos = $state<Memo[]>([]);
  let statusMap = $state<Record<string, string>>({});
  let connected = $state(false);

  // Editor state. `editingId` null = creating a new memo.
  let editingId = $state<string | null>(null);
  let title = $state("");
  let content = $state("");
  let isEvent = $state(false);
  let syncEnabled = $state(false);
  let busy = $state(false);
  let error = $state("");

  $effect(() => {
    // Reload whenever the selected date changes.
    void selectedDate;
    reload();
  });

  $effect(() => {
    googleStatus()
      .then((s) => (connected = s.connected))
      .catch(() => (connected = false));
  });

  async function reload() {
    try {
      memos = await listMemosByDate(selectedDate);
      statusMap = await syncStatusMap();
    } catch (e) {
      error = String(e);
    }
    resetForm();
  }

  function resetForm() {
    editingId = null;
    title = "";
    content = "";
    isEvent = false;
    syncEnabled = false;
    error = "";
  }

  function startEdit(m: Memo) {
    editingId = m.memo_id;
    title = m.title;
    content = m.content;
    isEvent = m.is_calendar_event;
    syncEnabled = m.sync_enabled;
    error = "";
  }

  async function save() {
    if (!title.trim()) {
      error = "제목을 입력하세요.";
      return;
    }
    busy = true;
    error = "";
    try {
      if (editingId) {
        const existing = memos.find((m) => m.memo_id === editingId)!;
        await updateMemo({
          ...existing,
          target_date: selectedDate,
          title: title.trim(),
          content,
          is_calendar_event: isEvent,
          sync_enabled: isEvent && syncEnabled,
        });
      } else {
        await createMemo({
          target_date: selectedDate,
          title: title.trim(),
          content,
          is_calendar_event: isEvent,
          sync_enabled: isEvent && syncEnabled,
        });
      }
      bumpMemos();
      await reload();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function remove(m: Memo) {
    busy = true;
    try {
      await deleteMemo(m.memo_id);
      bumpMemos();
      await reload();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function sync(m: Memo) {
    busy = true;
    error = "";
    try {
      await syncMemo(m.memo_id);
      statusMap = await syncStatusMap();
    } catch (e) {
      error = String(e);
      statusMap = await syncStatusMap();
    } finally {
      busy = false;
    }
  }
</script>

<section class="panel">
  <h3>{selectedDate}</h3>

  {#if memos.length === 0}
    <p class="muted">메모가 없습니다.</p>
  {:else}
    <ul class="memo-list">
      {#each memos as m (m.memo_id)}
        <li class:active={m.memo_id === editingId}>
          <button class="memo-row ghost" onclick={() => startEdit(m)}>
            <span class="memo-title">{m.title}</span>
            {#if m.is_calendar_event}<span class="badge ev">일정</span>{/if}
            {#if statusMap[m.memo_id] === "synced"}<span class="badge ok">동기화됨</span>
            {:else if statusMap[m.memo_id] === "failed"}<span class="badge fail">실패</span>{/if}
          </button>
          <div class="memo-actions">
            {#if connected && m.is_calendar_event && m.sync_enabled}
              <button class="ghost" title="Google에 반영" onclick={() => sync(m)} disabled={busy}>↑</button>
            {/if}
            <button class="ghost danger" title="삭제" onclick={() => remove(m)} disabled={busy}>🗑</button>
          </div>
        </li>
      {/each}
    </ul>
  {/if}

  <div class="editor">
    <input placeholder="제목" bind:value={title} />
    <textarea placeholder="내용" bind:value={content}></textarea>
    <label class="chk">
      <input type="checkbox" bind:checked={isEvent} /> 일정으로 지정 (ICS/Google 대상)
    </label>
    {#if isEvent}
      <label class="chk">
        <input type="checkbox" bind:checked={syncEnabled} /> Google Calendar 반영 대상
      </label>
    {/if}
    {#if error}<p class="err">{error}</p>{/if}
    <div class="editor-actions">
      <button class="primary" onclick={save} disabled={busy}>
        {editingId ? "수정" : "추가"}
      </button>
      {#if editingId}
        <button class="ghost" onclick={resetForm} disabled={busy}>취소</button>
      {/if}
    </div>
  </div>
</section>

<style>
  .panel {
    flex: 1;
    overflow-y: auto;
    padding: 8px 10px;
    border-top: 1px solid var(--border);
  }
  h3 {
    margin: 0 0 6px;
    font-size: 13px;
    color: var(--accent);
  }
  .muted {
    color: var(--muted);
    margin: 4px 0 10px;
  }
  .memo-list {
    list-style: none;
    margin: 0 0 10px;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .memo-list li {
    display: flex;
    align-items: center;
    gap: 4px;
    border-radius: 6px;
  }
  .memo-list li.active {
    outline: 1px solid var(--accent);
  }
  .memo-row {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 6px;
    text-align: left;
    background: var(--panel);
    border-color: var(--border);
    overflow: hidden;
  }
  .memo-title {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .memo-actions {
    display: flex;
    gap: 2px;
  }
  .badge {
    font-size: 10px;
    padding: 1px 5px;
    border-radius: 999px;
    background: var(--panel-2);
    color: var(--muted);
  }
  .badge.ev {
    background: #3a3d44;
    color: var(--accent);
  }
  .badge.ok {
    background: #234a2a;
    color: #8be29a;
  }
  .badge.fail {
    background: #4a2323;
    color: #f0a0a0;
  }
  .danger {
    color: #f0a0a0;
  }
  .editor {
    display: flex;
    flex-direction: column;
    gap: 6px;
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
  .editor-actions {
    display: flex;
    gap: 6px;
  }
  .err {
    color: #f0a0a0;
    margin: 0;
    font-size: 12px;
    word-break: break-word;
  }
</style>
