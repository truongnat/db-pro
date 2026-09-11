import { describe, it, expect } from "vitest";
import { LayoutCache, type CachedLayout } from "../utils/layout-cache";

/** Minimal in-memory Storage shim (jsdom's localStorage is not provided). */
function createFakeStorage(): Storage {
  const map = new Map<string, string>();
  return {
    get length() {
      return map.size;
    },
    clear: () => map.clear(),
    getItem: (key: string) => map.get(key) ?? null,
    key: (index: number) => [...map.keys()][index] ?? null,
    removeItem: (key: string) => {
      map.delete(key);
    },
    setItem: (key: string, value: string) => {
      map.set(key, value);
    },
  } as Storage;
}

function entry(hash: string, nodeIds: string[], createdAt = 1): CachedLayout {
  return {
    hash,
    positions: Object.fromEntries(nodeIds.map((id, i) => [id, { x: i * 10, y: i * 20 }])),
    layoutMs: 42,
    nodeIds,
    createdAt,
  };
}

describe("LayoutCache", () => {
  it("returns null on miss", () => {
    const cache = new LayoutCache(null);
    expect(cache.get("hash1", ["a", "b"])).toBeNull();
  });

  it("round-trips through memory", () => {
    const cache = new LayoutCache(null);
    cache.set(entry("h", ["a", "b"]));
    const got = cache.get("h", ["a", "b"]);
    expect(got?.positions.a).toEqual({ x: 0, y: 0 });
    expect(got?.positions.b).toEqual({ x: 10, y: 20 });
  });

  it("rejects a read whose node set differs (collision guard)", () => {
    const cache = new LayoutCache(null);
    cache.set(entry("h", ["a", "b"]));
    expect(cache.get("h", ["a", "c"])).toBeNull();
  });

  it("accepts an equal node set in different order", () => {
    const cache = new LayoutCache(null);
    cache.set(entry("h", ["a", "b"]));
    expect(cache.get("h", ["b", "a"])).not.toBeNull();
  });

  it("persists to storage and loads from it on a fresh instance", () => {
    const storage = createFakeStorage();
    const writer = new LayoutCache(storage);
    writer.set(entry("h", ["a", "b"]));

    const reader = new LayoutCache(storage);
    const got = reader.get("h", ["a", "b"]);
    expect(got?.positions.a).toEqual({ x: 0, y: 0 });
  });

  it("ignores corrupt storage entries", () => {
    const storage = createFakeStorage();
    storage.setItem("er-layout-cache:v1:bad", "{not json");
    const cache = new LayoutCache(storage);
    expect(cache.get("bad", ["a"])).toBeNull();
  });

  it("evicts the oldest persisted entry beyond maxEntries", () => {
    const storage = createFakeStorage();
    const cache = new LayoutCache(storage, 2);
    cache.set(entry("h1", ["a"], 100));
    cache.set(entry("h2", ["b"], 200));
    cache.set(entry("h3", ["c"], 300));

    // h1 is the oldest persisted entry → removed from storage (memory cache
    // keeps it for the session, which is fine — no session limit).
    expect(storage.getItem("er-layout-cache:v1:h1")).toBeNull();
    expect(storage.getItem("er-layout-cache:v1:h2")).not.toBeNull();
    expect(storage.getItem("er-layout-cache:v1:h3")).not.toBeNull();

    // A fresh cache (storage only) no longer sees h1.
    const fresh = new LayoutCache(storage, 2);
    expect(fresh.get("h1", ["a"])).toBeNull();
  });

  it("clear removes memory and storage entries", () => {
    const storage = createFakeStorage();
    const cache = new LayoutCache(storage);
    cache.set(entry("h1", ["a"]));
    cache.set(entry("h2", ["b"]));
    cache.clear();
    expect(cache.get("h1", ["a"])).toBeNull();
    expect(cache.get("h2", ["b"])).toBeNull();
    expect(storage.length).toBe(0);
  });
});
