import { beforeEach, describe, expect, it } from "vitest";

import { useQueryHistoryStore } from "@/commons/stores/query-history.store";

function makeEntry(overrides: Record<string, unknown> = {}) {
  return {
    id: "h1",
    connectionId: "c1",
    sql: "SELECT 1",
    executedAt: new Date().toISOString(),
    durationMs: 42,
    rowCount: 1,
    status: "success" as const,
    database: null,
    schema: null,
    ...overrides,
  };
}

function resetStore() {
  useQueryHistoryStore.getState().clearHistory();
  useQueryHistoryStore.setState({ favorites: new Set() });
}

describe("QueryHistoryStore (enhanced)", () => {
  beforeEach(() => {
    resetStore();
  });

  it("adds entry with status and context", () => {
    useQueryHistoryStore
      .getState()
      .addEntry(makeEntry({ status: "success", database: "mydb", schema: "public" }));
    const entries = useQueryHistoryStore.getState().entries;
    expect(entries).toHaveLength(1);
    expect(entries[0].status).toBe("success");
    expect(entries[0].database).toBe("mydb");
    expect(entries[0].schema).toBe("public");
  });

  it("adds error entry", () => {
    useQueryHistoryStore.getState().addEntry(makeEntry({ status: "error" }));
    expect(useQueryHistoryStore.getState().entries[0].status).toBe("error");
  });

  it("toggles favorite on", () => {
    useQueryHistoryStore.getState().addEntry(makeEntry({ id: "h1" }));
    useQueryHistoryStore.getState().toggleFavorite("h1");
    expect(useQueryHistoryStore.getState().isFavorite("h1")).toBe(true);
  });

  it("toggles favorite off", () => {
    useQueryHistoryStore.getState().toggleFavorite("h1");
    useQueryHistoryStore.getState().toggleFavorite("h1");
    expect(useQueryHistoryStore.getState().isFavorite("h1")).toBe(false);
  });

  it("clears history but preserves favorites", () => {
    useQueryHistoryStore.getState().addEntry(makeEntry({ id: "h1" }));
    useQueryHistoryStore.getState().toggleFavorite("h1");
    useQueryHistoryStore.getState().clearHistory();
    expect(useQueryHistoryStore.getState().entries).toHaveLength(0);
    expect(useQueryHistoryStore.getState().isFavorite("h1")).toBe(true);
  });

  it("clears by connection", () => {
    useQueryHistoryStore.getState().addEntry(makeEntry({ id: "h1", connectionId: "c1" }));
    useQueryHistoryStore.getState().addEntry(makeEntry({ id: "h2", connectionId: "c2" }));
    useQueryHistoryStore.getState().clearByConnection("c1");
    const entries = useQueryHistoryStore.getState().entries;
    expect(entries).toHaveLength(1);
    expect(entries[0].connectionId).toBe("c2");
  });
});
