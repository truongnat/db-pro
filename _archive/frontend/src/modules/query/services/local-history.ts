const STORAGE_KEY = "db-pro-query-local-history";
const MAX_ENTRIES = 50;

export interface LocalHistoryEntry {
  sql: string;
  timestamp: number;
}

export function getLocalHistory(): LocalHistoryEntry[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    return JSON.parse(raw) as LocalHistoryEntry[];
  } catch {
    return [];
  }
}

export function pushLocalHistory(sql: string): void {
  const trimmed = sql.trim();
  if (!trimmed) return;

  const entries = getLocalHistory();
  const now = Date.now();

  if (entries.length > 0 && entries[0].sql === trimmed) {
    entries[0].timestamp = now;
  } else {
    entries.unshift({ sql: trimmed, timestamp: now });
  }

  while (entries.length > MAX_ENTRIES) {
    entries.pop();
  }

  localStorage.setItem(STORAGE_KEY, JSON.stringify(entries));
}

export function clearLocalHistory(): void {
  localStorage.removeItem(STORAGE_KEY);
}

export function removeLocalHistoryEntry(index: number): void {
  const entries = getLocalHistory();
  entries.splice(index, 1);
  localStorage.setItem(STORAGE_KEY, JSON.stringify(entries));
}
