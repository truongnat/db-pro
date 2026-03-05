import type { ConnectionDraft } from "./draft";

const DRAFT_CACHE_KEY = "dbpro.connection.draft.v1";
const SELECTED_CONNECTION_CACHE_KEY = "dbpro.connection.selected.v1";
const QUERY_PAGE_SIZE_CACHE_KEY = "dbpro.query.pageSize.v1";

function readLocalStorage(key: string): string | null {
  try {
    return window.localStorage.getItem(key);
  } catch {
    return null;
  }
}

function writeLocalStorage(key: string, value: string): void {
  try {
    window.localStorage.setItem(key, value);
  } catch {
    // Ignore storage errors in sandboxed/private contexts.
  }
}

function removeLocalStorage(key: string): void {
  try {
    window.localStorage.removeItem(key);
  } catch {
    // Ignore storage errors in sandboxed/private contexts.
  }
}

export function loadCachedDraft(fallback: ConnectionDraft): ConnectionDraft {
  const raw = readLocalStorage(DRAFT_CACHE_KEY);
  if (!raw) {
    return fallback;
  }

  try {
    const parsed = JSON.parse(raw) as Partial<ConnectionDraft>;
    return {
      ...fallback,
      ...parsed,
    };
  } catch {
    return fallback;
  }
}

export function saveCachedDraft(draft: ConnectionDraft): void {
  writeLocalStorage(DRAFT_CACHE_KEY, JSON.stringify(draft));
}

export function resetCachedDraft(): void {
  removeLocalStorage(DRAFT_CACHE_KEY);
}

export function loadCachedSelectedConnectionId(): string | null {
  return readLocalStorage(SELECTED_CONNECTION_CACHE_KEY);
}

export function saveCachedSelectedConnectionId(connectionId: string | null): void {
  if (!connectionId) {
    removeLocalStorage(SELECTED_CONNECTION_CACHE_KEY);
    return;
  }
  writeLocalStorage(SELECTED_CONNECTION_CACHE_KEY, connectionId);
}

export function loadCachedQueryPageSize(defaultValue: number): number {
  const raw = readLocalStorage(QUERY_PAGE_SIZE_CACHE_KEY);
  if (!raw) {
    return defaultValue;
  }

  const parsed = Number.parseInt(raw, 10);
  if (!Number.isFinite(parsed) || parsed <= 0) {
    return defaultValue;
  }

  return parsed;
}

export function saveCachedQueryPageSize(pageSize: number): void {
  writeLocalStorage(QUERY_PAGE_SIZE_CACHE_KEY, pageSize.toString());
}

export function clearConnectionPanelCache(): void {
  resetCachedDraft();
  removeLocalStorage(SELECTED_CONNECTION_CACHE_KEY);
  removeLocalStorage(QUERY_PAGE_SIZE_CACHE_KEY);
}
