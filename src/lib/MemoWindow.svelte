<script lang="ts">
  import { onMount } from "svelte";
  import { listen, emit } from "@tauri-apps/api/event";
  import MemoPanel from "./MemoPanel.svelte";
  import { getViewDate, getSetting } from "./api";
  import { bus } from "./store.svelte";

  let current = $state("");

  onMount(() => {
    getViewDate("memo").then((d) => {
      if (d) current = d;
    });
    applyScale();
    // The main window may reopen this for a different date.
    const un = listen<string>("memo-date", (e) => (current = e.payload));
    return () => {
      un.then((f) => f());
    };
  });

  async function applyScale() {
    const s = await getSetting("ui_scale");
    const n = s ? Number(s) : NaN;
    if (!Number.isNaN(n)) document.documentElement.style.setProperty("zoom", String(n));
  }

  // Bridge local memo changes to the main window so the calendar refreshes.
  let lastVersion = bus.memoVersion;
  $effect(() => {
    if (bus.memoVersion !== lastVersion) {
      lastVersion = bus.memoVersion;
      emit("memos-changed");
    }
  });
</script>

{#if current}
  <MemoPanel selectedDate={current} />
{/if}
