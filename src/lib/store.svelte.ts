import type { WindowMode } from "./api";

// Tiny reactive event bus so the calendar re-fetches when memos change.
export const bus = $state({ memoVersion: 0 });

export function bumpMemos(): void {
  bus.memoVersion++;
}

// Shared UI state for window appearance, loaded once at startup and mutated
// from Settings. App reacts to these (titlebar visibility, opacity).
export const ui = $state<{ windowMode: WindowMode; opacity: number; scale: number }>({
  windowMode: "normal",
  opacity: 0.85,
  scale: 1,
});
