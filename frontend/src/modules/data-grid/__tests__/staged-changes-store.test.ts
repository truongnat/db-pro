import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { useStagedChangesStore, type StagedCellEdit } from "../state/staged-changes.store";
import type { CellValue } from "@/modules/query/types/query.types";

function resetStore() {
  useStagedChangesStore.setState({ changes: {} });
}

function makeCellEdit(overrides: Partial<StagedCellEdit> = {}): StagedCellEdit {
  return {
    kind: "cell-edit",
    pkValues: [{ type: "int64", value: 1 }],
    currentValues: [{ type: "int64", value: 1 }, { type: "text", value: "Alice" }],
    columnName: "name",
    newValue: { type: "text", value: "Bob" },
    ...overrides,
  };
}

/** Helper: always read fresh state after mutations. */
function changes(tabId: string) {
  return useStagedChangesStore.getState().changes[tabId];
}

describe("StagedChangesStore", () => {
  beforeEach(resetStore);
  afterEach(resetStore);

  describe("stageCellEdit", () => {
    it("stages a cell edit", () => {
      const edit = makeCellEdit();
      useStagedChangesStore.getState().stageCellEdit("tab-1", edit);
      const c = changes("tab-1");
      expect(c).toHaveLength(1);
      expect(c![0].kind).toBe("cell-edit");
    });

    it("replaces previous edit for same row + column", () => {
      const edit1 = makeCellEdit({ newValue: { type: "text", value: "Bob" } });
      const edit2 = makeCellEdit({ newValue: { type: "text", value: "Carol" } });
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", edit1);
      store.stageCellEdit("tab-1", edit2);
      const c = changes("tab-1");
      expect(c).toHaveLength(1);
      expect((c![0] as StagedCellEdit).newValue).toEqual({ type: "text", value: "Carol" });
    });

    it("keeps edits for different columns", () => {
      const edit1 = makeCellEdit({ columnName: "name" });
      const edit2 = makeCellEdit({ columnName: "email" });
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", edit1);
      store.stageCellEdit("tab-1", edit2);
      expect(changes("tab-1")).toHaveLength(2);
    });

    it("keeps edits for different rows", () => {
      const edit1 = makeCellEdit({ pkValues: [{ type: "int64", value: 1 }] });
      const edit2 = makeCellEdit({ pkValues: [{ type: "int64", value: 2 }] });
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", edit1);
      store.stageCellEdit("tab-1", edit2);
      expect(changes("tab-1")).toHaveLength(2);
    });

    it("isolates changes per tab", () => {
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", makeCellEdit());
      store.stageCellEdit("tab-2", makeCellEdit({ columnName: "email" }));
      expect(changes("tab-1")).toHaveLength(1);
      expect(changes("tab-2")).toHaveLength(1);
    });
  });

  describe("stageDeleteRow", () => {
    it("stages a row deletion", () => {
      const pk: CellValue[] = [{ type: "int64", value: 1 }];
      useStagedChangesStore.getState().stageDeleteRow("tab-1", pk);
      const c = changes("tab-1");
      expect(c).toHaveLength(1);
      expect(c![0].kind).toBe("row-delete");
    });

    it("removes cell edits for the same row when deleting", () => {
      const pk: CellValue[] = [{ type: "int64", value: 1 }];
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", makeCellEdit({ pkValues: pk }));
      expect(changes("tab-1")).toHaveLength(1);
      store.stageDeleteRow("tab-1", pk);
      const c = changes("tab-1");
      expect(c).toHaveLength(1);
      expect(c![0].kind).toBe("row-delete");
    });

    it("does not remove cell edits for different rows", () => {
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", makeCellEdit({ pkValues: [{ type: "int64", value: 1 }] }));
      store.stageDeleteRow("tab-1", [{ type: "int64", value: 2 }]);
      expect(changes("tab-1")).toHaveLength(2);
    });

    it("replaces duplicate delete for same row", () => {
      const pk: CellValue[] = [{ type: "int64", value: 1 }];
      const store = useStagedChangesStore.getState();
      store.stageDeleteRow("tab-1", pk);
      store.stageDeleteRow("tab-1", pk);
      expect(changes("tab-1")).toHaveLength(1);
    });
  });

  describe("revertAt", () => {
    it("removes change at index", () => {
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", makeCellEdit({ columnName: "name" }));
      store.stageCellEdit("tab-1", makeCellEdit({ columnName: "email" }));
      store.revertAt("tab-1", 0);
      const c = changes("tab-1");
      expect(c).toHaveLength(1);
      expect((c![0] as StagedCellEdit).columnName).toBe("email");
    });

    it("handles out-of-bounds index gracefully", () => {
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", makeCellEdit());
      store.revertAt("tab-1", 5);
      expect(changes("tab-1")).toHaveLength(1);
      store.revertAt("tab-1", -1);
      expect(changes("tab-1")).toHaveLength(1);
    });

    it("handles unknown tab gracefully", () => {
      useStagedChangesStore.getState().revertAt("unknown", 0);
      // Should not throw
    });
  });

  describe("revertAll", () => {
    it("removes all changes for a tab", () => {
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", makeCellEdit());
      store.stageCellEdit("tab-1", makeCellEdit({ columnName: "email" }));
      store.revertAll("tab-1");
      expect(changes("tab-1")).toBeUndefined();
    });

    it("does not affect other tabs", () => {
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", makeCellEdit());
      store.stageCellEdit("tab-2", makeCellEdit());
      store.revertAll("tab-1");
      expect(changes("tab-2")).toHaveLength(1);
    });
  });

  describe("clearChanges", () => {
    it("clears changes for a tab (same as revertAll)", () => {
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", makeCellEdit());
      store.clearChanges("tab-1");
      expect(changes("tab-1")).toBeUndefined();
    });
  });

  describe("gc", () => {
    it("removes changes for tabs no longer open", () => {
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", makeCellEdit());
      store.stageCellEdit("tab-2", makeCellEdit());
      store.stageCellEdit("tab-3", makeCellEdit());
      store.gc(new Set(["tab-1", "tab-3"]));
      const state = useStagedChangesStore.getState();
      expect(state.changes["tab-1"]).toBeDefined();
      expect(state.changes["tab-2"]).toBeUndefined();
      expect(state.changes["tab-3"]).toBeDefined();
    });

    it("handles empty valid set", () => {
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", makeCellEdit());
      store.gc(new Set());
      expect(Object.keys(useStagedChangesStore.getState().changes)).toHaveLength(0);
    });
  });
});
