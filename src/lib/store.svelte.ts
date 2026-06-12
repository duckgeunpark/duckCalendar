// Tiny reactive event bus so the calendar re-fetches when memos change.
export const bus = $state({ memoVersion: 0 });

export function bumpMemos(): void {
  bus.memoVersion++;
}
