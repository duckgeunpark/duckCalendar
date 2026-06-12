<script lang="ts">
  import { listMemosByDate, openMemoWindow } from "./api";
  import type { Memo } from "./types";
  import { bus } from "./store.svelte";
  import { WEEKDAYS } from "./date";

  interface Props {
    date: string; // 'YYYY-MM-DD'
    onback: () => void;
  }
  let { date, onback }: Props = $props();

  let memos = $state<Memo[]>([]);

  $effect(() => {
    void date;
    void bus.memoVersion;
    listMemosByDate(date)
      .then((m) => (memos = m))
      .catch((e) => console.error("listMemosByDate failed", e));
  });

  let weekday = $derived(WEEKDAYS[new Date(date + "T00:00:00").getDay()]);
</script>

<section class="day">
  <div class="day-head">
    <strong>{date} ({weekday})</strong>
    <button class="ghost back" onclick={onback} title="월별 보기 (우클릭으로도 가능)">월별 ▦</button>
  </div>

  <button class="primary add" onclick={() => openMemoWindow(date)}>＋ 메모 추가 / 편집</button>

  {#if memos.length === 0}
    <p class="muted">메모가 없습니다. 위 버튼으로 추가하세요.</p>
  {:else}
    <ul class="list">
      {#each memos as m (m.memo_id)}
        <li>
          <button class="ghost item" onclick={() => openMemoWindow(date)}>
            <span class="t">{m.title}{#if m.is_calendar_event} 📌{/if}</span>
            {#if m.content}<span class="c">{m.content}</span>{/if}
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
