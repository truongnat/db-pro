const QUERY_HISTORY_CACHE_KEY = "dbpro.query.history.v1";
const QUERY_HISTORY_LIMIT = 200;

export type QueryHistoryEntry = {
  id: string;
  connectionId: string;
  connectionName: string;
  sql: string;
  runSource: string;
  executionMs: number;
  recordedAt: string;
};

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
    // Ignore storage write failures in restricted environments.
  }
}

function sanitizeEntry(input: unknown): QueryHistoryEntry | null {
  if (!input || typeof input !== "object") {
    return null;
  }

  const entry = input as Partial<QueryHistoryEntry>;
  if (
    typeof entry.id !== "string" ||
    typeof entry.connectionId !== "string" ||
    typeof entry.connectionName !== "string" ||
    typeof entry.sql !== "string" ||
    typeof entry.runSource !== "string" ||
    typeof entry.recordedAt !== "string"
  ) {
    return null;
  }

  const executionMs = Number.isFinite(entry.executionMs)
    ? Number(entry.executionMs)
    : 0;

  return {
    id: entry.id,
    connectionId: entry.connectionId,
    connectionName: entry.connectionName,
    sql: entry.sql,
    runSource: entry.runSource,
    executionMs,
    recordedAt: entry.recordedAt,
  };
}

export function loadQueryHistory(): QueryHistoryEntry[] {
  const raw = readLocalStorage(QUERY_HISTORY_CACHE_KEY);
  if (!raw) {
    return [];
  }

  try {
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) {
      return [];
    }

    return parsed
      .map((entry) => sanitizeEntry(entry))
      .filter((entry): entry is QueryHistoryEntry => entry !== null)
      .slice(0, QUERY_HISTORY_LIMIT);
  } catch {
    return [];
  }
}

export function saveQueryHistory(entries: QueryHistoryEntry[]): void {
  writeLocalStorage(
    QUERY_HISTORY_CACHE_KEY,
    JSON.stringify(entries.slice(0, QUERY_HISTORY_LIMIT)),
  );
}

export function appendQueryHistory(
  entries: QueryHistoryEntry[],
  nextEntry: QueryHistoryEntry,
): QueryHistoryEntry[] {
  const normalizedSql = nextEntry.sql.trim();
  const filtered = entries.filter(
    (entry) =>
      !(
        entry.connectionId === nextEntry.connectionId &&
        entry.sql.trim() === normalizedSql
      ),
  );

  return [nextEntry, ...filtered].slice(0, QUERY_HISTORY_LIMIT);
}

export function clearQueryHistory(): void {
  writeLocalStorage(QUERY_HISTORY_CACHE_KEY, "[]");
}
