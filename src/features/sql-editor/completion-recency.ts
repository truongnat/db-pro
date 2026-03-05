const RECENCY_STORAGE_KEY = "dbpro.sql.completionRecency.v1";
const RECENCY_MAX_ENTRIES = 300;
const RECENCY_STALE_WINDOW_MS = 1000 * 60 * 60 * 24 * 30;
const RECENCY_FRESH_WINDOW_MS = 1000 * 60 * 60 * 24 * 14;

type CompletionUsageEntry = {
  key: string;
  label: string;
  type: string;
  count: number;
  lastPickedAt: number;
};

const usageByKey = new Map<string, CompletionUsageEntry>();
let usageLoaded = false;

export function recordCompletionUsage(label: string, type?: string) {
  if (!label.trim()) {
    return;
  }

  ensureUsageLoaded();
  const now = Date.now();
  const normalizedType = normalizeType(type);
  const key = buildUsageKey(label, normalizedType);
  const previous = usageByKey.get(key);

  usageByKey.set(key, {
    key,
    label: label.trim(),
    type: normalizedType,
    count: Math.min((previous?.count ?? 0) + 1, 99),
    lastPickedAt: now,
  });

  pruneUsage(now);
  persistUsage();
}

export function getCompletionRecencyBoost(
  label: string,
  type?: string,
  now = Date.now(),
): number {
  ensureUsageLoaded();

  const key = buildUsageKey(label, normalizeType(type));
  const entry = usageByKey.get(key);
  if (!entry) {
    return 0;
  }

  const ageMs = Math.max(0, now - entry.lastPickedAt);
  const freshness = Math.max(0, 1 - ageMs / RECENCY_FRESH_WINDOW_MS);
  const countBoost = Math.min(18, entry.count * 2);
  const freshnessBoost = Math.round(freshness * 12);

  return countBoost + freshnessBoost;
}

function buildUsageKey(label: string, type: string): string {
  return `${normalizeLabel(label)}::${type}`;
}

function normalizeLabel(value: string): string {
  return value.trim().replace(/"/g, "").toLowerCase();
}

function normalizeType(value: string | undefined): string {
  if (!value) {
    return "text";
  }

  const firstToken = value.trim().split(/\s+/)[0];
  return firstToken || "text";
}

function ensureUsageLoaded() {
  if (usageLoaded) {
    return;
  }
  usageLoaded = true;

  if (!canUseStorage()) {
    return;
  }

  try {
    const raw = window.localStorage.getItem(RECENCY_STORAGE_KEY);
    if (!raw) {
      return;
    }

    const parsed = JSON.parse(raw) as CompletionUsageEntry[];
    if (!Array.isArray(parsed)) {
      return;
    }

    for (const entry of parsed) {
      if (
        !entry ||
        typeof entry !== "object" ||
        typeof entry.key !== "string" ||
        typeof entry.label !== "string" ||
        typeof entry.type !== "string" ||
        typeof entry.count !== "number" ||
        typeof entry.lastPickedAt !== "number"
      ) {
        continue;
      }

      usageByKey.set(entry.key, entry);
    }

    pruneUsage(Date.now());
  } catch {
    usageByKey.clear();
  }
}

function pruneUsage(now: number) {
  for (const [key, entry] of usageByKey.entries()) {
    if (now - entry.lastPickedAt > RECENCY_STALE_WINDOW_MS) {
      usageByKey.delete(key);
    }
  }

  if (usageByKey.size <= RECENCY_MAX_ENTRIES) {
    return;
  }

  const sortedByRecent = Array.from(usageByKey.values()).sort(
    (left, right) => right.lastPickedAt - left.lastPickedAt,
  );
  const keep = new Set(
    sortedByRecent.slice(0, RECENCY_MAX_ENTRIES).map((entry) => entry.key),
  );

  for (const key of usageByKey.keys()) {
    if (!keep.has(key)) {
      usageByKey.delete(key);
    }
  }
}

function persistUsage() {
  if (!canUseStorage()) {
    return;
  }

  try {
    const payload = Array.from(usageByKey.values()).sort(
      (left, right) => right.lastPickedAt - left.lastPickedAt,
    );
    window.localStorage.setItem(RECENCY_STORAGE_KEY, JSON.stringify(payload));
  } catch {
    // Ignore storage failures (private mode/full quota), ranking still works in-memory.
  }
}

function canUseStorage(): boolean {
  return typeof window !== "undefined" && typeof window.localStorage !== "undefined";
}
