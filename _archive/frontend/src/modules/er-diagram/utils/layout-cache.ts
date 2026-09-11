import type { LayoutPosition } from "./layout";

/** A persisted layout result, keyed by `computeLayoutHash`. */
export interface CachedLayout {
  hash: string;
  /** nodeId → position (serializable form of the Map). */
  positions: Record<string, LayoutPosition>;
  layoutMs: number;
  /** Exact node-id set this layout was computed for — integrity check on read. */
  nodeIds: string[];
  createdAt: number;
}

const STORAGE_PREFIX = "er-layout-cache:v1:";
const DEFAULT_MAX_ENTRIES = 20;

/**
 * `schemaHash → positions` cache from the locked P1 architecture.
 *
 * - In-memory Map for the session (fast hit on direction toggles / scope changes).
 * - localStorage for cross-session reuse — opening a 1,000-table schema again
 *   skips the (measured) 122 s dagre run entirely.
 * - `get()` verifies the exact node-id set matches, so a hash collision can
 *   never serve positions for the wrong graph.
 * - Beyond `maxEntries` persisted layouts, the oldest entries are evicted.
 */
export class LayoutCache {
  private memory = new Map<string, CachedLayout>();
  private storage: Storage | null;
  private maxEntries: number;

  constructor(
    storage: Storage | null = typeof localStorage !== "undefined" ? localStorage : null,
    maxEntries = DEFAULT_MAX_ENTRIES,
  ) {
    this.storage = storage;
    this.maxEntries = maxEntries;
  }

  get(hash: string, nodeIds: string[]): CachedLayout | null {
    const mem = this.memory.get(hash);
    if (mem) {
      if (sameNodeSet(mem.nodeIds, nodeIds)) return mem;
      this.memory.delete(hash); // collision with a different graph
      return null;
    }
    if (!this.storage) return null;
    const raw = this.storage.getItem(STORAGE_PREFIX + hash);
    if (!raw) return null;
    try {
      const parsed = JSON.parse(raw) as CachedLayout;
      if (!sameNodeSet(parsed.nodeIds, nodeIds)) return null;
      this.memory.set(hash, parsed);
      return parsed;
    } catch {
      return null;
    }
  }

  set(entry: CachedLayout): void {
    this.memory.set(entry.hash, entry);
    if (!this.storage) return;
    try {
      this.storage.setItem(STORAGE_PREFIX + entry.hash, JSON.stringify(entry));
      this.evictOldest();
    } catch {
      // Quota exceeded or storage unavailable — memory cache still works.
    }
  }

  clear(): void {
    this.memory.clear();
    if (!this.storage) return;
    const toRemove: string[] = [];
    for (let i = 0; i < this.storage.length; i++) {
      const key = this.storage.key(i);
      if (key?.startsWith(STORAGE_PREFIX)) toRemove.push(key);
    }
    for (const key of toRemove) this.storage.removeItem(key);
  }

  private evictOldest(): void {
    if (!this.storage) return;
    const entries: { key: string; createdAt: number }[] = [];
    for (let i = 0; i < this.storage.length; i++) {
      const key = this.storage.key(i);
      if (!key?.startsWith(STORAGE_PREFIX)) continue;
      const raw = this.storage.getItem(key);
      if (!raw) continue;
      try {
        entries.push({ key, createdAt: (JSON.parse(raw) as CachedLayout).createdAt ?? 0 });
      } catch {
        this.storage.removeItem(key);
      }
    }
    if (entries.length <= this.maxEntries) return;
    entries.sort((a, b) => a.createdAt - b.createdAt);
    for (const oldest of entries.slice(0, entries.length - this.maxEntries)) {
      this.storage.removeItem(oldest.key);
    }
  }
}

function sameNodeSet(a: string[], b: string[]): boolean {
  if (a.length !== b.length) return false;
  const sa = [...a].sort();
  const sb = [...b].sort();
  return sa.every((v, i) => v === sb[i]);
}
