import { mount } from "svelte";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./app.css";
import App from "./App.svelte";
import MemoWindow from "./lib/MemoWindow.svelte";
import SettingsWindow from "./lib/SettingsWindow.svelte";

const label = getCurrentWindow().label;
const target = document.getElementById("app")!;

// Child windows render opaque, regardless of the main widget's opacity.
if (label !== "main") {
  document.documentElement.style.setProperty("--app-alpha", "1");
}

let app;
if (label === "memo") {
  app = mount(MemoWindow, { target });
} else if (label === "settings") {
  app = mount(SettingsWindow, { target });
} else {
  app = mount(App, { target });
}

export default app;
