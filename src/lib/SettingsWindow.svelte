<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import Settings from "./Settings.svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { getViewDate } from "./api";

  let date = $state("");
  const appWindow = getCurrentWindow();

  onMount(() => {
    getViewDate("settings").then((d) => {
      date = d || new Date().toISOString().slice(0, 10);
    });
    const un = listen<string>("settings-date", (e) => {
      if (e.payload) date = e.payload;
    });
    return () => {
      un.then((f) => f());
    };
  });
</script>

{#if date}
  <Settings selectedDate={date} onclose={() => appWindow.close()} />
{/if}
