import type { QueryTab } from "../state/query.store";

const STORAGE_PREFIX = "db-pro-query-tabs";
const SAVE_DEBOUNCE_MS = 1000;

interface PersistedState {
  tabs: QueryTab[];
  activeTabId: string;
}

function storageKey(connectionId: string): string {
  return `${STORAGE_PREFIX}:${connectionId}`;
}

export function saveTabs(connectionId: string, tabs: QueryTab[], activeTabId: string): void {
  const state: PersistedState = {
    tabs: tabs.map((t) => ({
      ...t,
      result: null,
      explainPlan: null,
      status: "idle",
      error: null,
    })),
    activeTabId,
  };
  try {
    localStorage.setItem(storageKey(connectionId), JSON.stringify(state));
  } catch {
    // storage full or unavailable
  }
}

export function loadTabs(connectionId: string): PersistedState | null {
  try {
    const raw = localStorage.getItem(storageKey(connectionId));
    if (!raw) return null;
    return JSON.parse(raw) as PersistedState;
  } catch {
    return null;
  }
}

export function clearTabs(connectionId: string): void {
  localStorage.removeItem(storageKey(connectionId));
}

let debounceTimer: ReturnType<typeof setTimeout> | null = null;

export function debouncedSaveTabs(
  connectionId: string,
  tabs: QueryTab[],
  activeTabId: string,
): void {
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => {
    saveTabs(connectionId, tabs, activeTabId);
  }, SAVE_DEBOUNCE_MS);
}
