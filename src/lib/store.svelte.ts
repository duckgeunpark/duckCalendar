import type { WindowMode } from "./api";

// Tiny reactive event bus so the calendar re-fetches when memos change.
export const bus = $state({ memoVersion: 0 });

export function bumpMemos(): void {
  bus.memoVersion++;
}

// Shared UI state for window appearance, loaded once at startup and mutated
// from Settings. App reacts to these (titlebar visibility, opacity).
export type ThemeName = "dark" | "light" | "navy" | "sepia";
export type Lang = "ko" | "en";

export const ui = $state<{
  windowMode: WindowMode;
  opacity: number;
  scale: number;
  /** Large "자세히보기" (expanded) layout vs the compact "간략보기" widget. */
  expanded: boolean;
  /** Preset color theme (applied as data-theme on <html>). */
  theme: ThemeName;
  /** Optional accent color override (hex); empty = use the theme's accent. */
  accent: string;
  /** UI language. */
  lang: Lang;
}>({
  windowMode: "normal",
  opacity: 0.85,
  scale: 1,
  expanded: false,
  theme: "dark",
  accent: "",
  lang: "ko",
});
