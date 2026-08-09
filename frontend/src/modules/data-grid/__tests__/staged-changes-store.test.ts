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

  describe("markFailed", () => {
    it("marks specific indices with error", () => {
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", makeCellEdit({ columnName: "a" }));
      store.stageCellEdit("tab-1", makeCellEdit({ columnName: "b" }));
      store.stageCellEdit("tab-1", makeCellEdit({ columnName: "c" }));
      store.markFailed("tab-1", [1], "DB error");
      const c = changes("tab-1");
      expect(c).toHaveLength(3);
      expect(c![0].error).toBeUndefined();
      expect(c![1].error).toBe("DB error");
      expect(c![2].error).toBeUndefined();
    });
  });

  describe("keepOnlyIndices (partial apply correctness)", () => {
    it("removes successful changes, keeps only failed ones", () => {
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", makeCellEdit({ columnName: "a" }));
      store.stageCellEdit("tab-1", makeCellEdit({ columnName: "b" }));
      store.stageCellEdit("tab-1", makeCellEdit({ columnName: "c" }));

      // Simulate partial success: indices 0,1 succeeded, index 2 failed
      store.keepOnlyIndices("tab-1", [2]);
      const c = changes("tab-1");
      expect(c).toHaveLength(1);
      expect((c![0] as StagedCellEdit).columnName).toBe("c");
    });

    it("handles keeping multiple failed indices", () => {
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", makeCellEdit({ columnName: "a" }));
      store.stageDeleteRow("tab-1", [{ type: "int64", value: 99 }]);
      store.stageCellEdit("tab-1", makeCellEdit({ columnName: "c" }));

      // Index 0 succeeded, indices 1,2 failed
      store.keepOnlyIndices("tab-1", [1, 2]);
      const c = changes("tab-1");
      expect(c).toHaveLength(2);
      expect(c![0].kind).toBe("row-delete");
      expect((c![1] as StagedCellEdit).columnName).toBe("c");
    });

    it("handles empty keep set (all succeeded)", () => {
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", makeCellEdit());
      store.keepOnlyIndices("tab-1", []);
      expect(changes("tab-1")).toHaveLength(0);
    });

    it("handles unknown tab gracefully", () => {
      useStagedChangesStore.getState().keepOnlyIndices("unknown", [0]);
      // Should not throw
    });
  });

  describe("removeByIds (identity-safe apply)", () => {
    it("removes only changes with matching IDs", () => {
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", makeCellEdit({ columnName: "a" }));
      store.stageCellEdit("tab-1", makeCellEdit({ columnName: "b" }));
      store.stageCellEdit("tab-1", makeCellEdit({ columnName: "c" }));
      const allIds = changes("tab-1")!.map((c) => c.id);

      // Remove first two by ID
      store.removeByIds("tab-1", [allIds[0], allIds[1]]);
      const c = changes("tab-1");
      expect(c).toHaveLength(1);
      expect((c![0] as StagedCellEdit).columnName).toBe("c");
    });

    it("preserves changes staged during apply (concurrency safety)", () => {
      const store = useStagedChangesStore.getState();
      // Initial: stage A
      store.stageCellEdit("tab-1", makeCellEdit({ columnName: "a" }));
      const idA = changes("tab-1")![0].id;

      // While A is "in flight", user stages B
      store.stageCellEdit("tab-1", makeCellEdit({ columnName: "b" }));

      // A succeeds → removeByIds only removes A's ID
      store.removeByIds("tab-1", [idA]);

      const c = changes("tab-1");
      expect(c).toHaveLength(1);
      expect((c![0] as StagedCellEdit).columnName).toBe("b");
    });

    it("handles retry-failed with pending changes safely", () => {
      const store = useStagedChangesStore.getState();
      // A failed, B is new pending
      store.stageCellEdit("tab-1", makeCellEdit({ columnName: "a" }));
      store.markFailed("tab-1", [0], "DB error");
      store.stageCellEdit("tab-1", makeCellEdit({ columnName: "b" }));

      const idA = changes("tab-1")![0].id;
      expect(changes("tab-1")).toHaveLength(2);

      // Retry Failed: only A is retried, A succeeds
      store.removeByIds("tab-1", [idA]);

      const c = changes("tab-1");
      expect(c).toHaveLength(1);
      expect((c![0] as StagedCellEdit).columnName).toBe("b");
      expect(c![0].error).toBeUndefined(); // B was never touched
    });

    it("partial retry: A(error) B(error) C(pending) → A success B fail", () => {
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", makeCellEdit({ columnName: "a" }));
      store.stageCellEdit("tab-1", makeCellEdit({ columnName: "b" }));
      store.stageCellEdit("tab-1", makeCellEdit({ columnName: "c" }));
      store.markFailed("tab-1", [0, 1], "fail");

      const all = changes("tab-1")!;
      const idA = all[0].id;
      const idB = all[1].id;

      // Retry Failed: A and B retried. A succeeds, B fails again.
      store.removeByIds("tab-1", [idA]); // A succeeded
      store.markFailedByIds("tab-1", [{ id: idB, error: "still failing" }]);

      const c = changes("tab-1");
      expect(c).toHaveLength(2);
      // B should have error, C should be pending (no error)
      const bChange = c!.find((ch) => (ch as StagedCellEdit).columnName === "b");
      const cChange = c!.find((ch) => (ch as StagedCellEdit).columnName === "c");
      expect(bChange!.error).toBe("still failing");
      expect(cChange!.error).toBeUndefined();
    });

    it("handles unknown tab gracefully", () => {
      useStagedChangesStore.getState().removeByIds("unknown", ["x"]);
      // Should not throw
    });
  });

  describe("markFailedByIds", () => {
    it("marks specific changes by ID with error", () => {
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", makeCellEdit({ columnName: "a" }));
      store.stageCellEdit("tab-1", makeCellEdit({ columnName: "b" }));
      const ids = changes("tab-1")!.map((c) => c.id);

      store.markFailedByIds("tab-1", [{ id: ids[1], error: "DB error" }]);
      const c = changes("tab-1");
      expect(c![0].error).toBeUndefined();
      expect(c![1].error).toBe("DB error");
    });

    it("ignores unknown IDs", () => {
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", makeCellEdit({ columnName: "a" }));
      store.markFailedByIds("tab-1", [{ id: "nonexistent", error: "x" }]);
      expect(changes("tab-1")![0].error).toBeUndefined();
    });
  });

  describe("revertById", () => {
    it("removes change by ID", () => {
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", makeCellEdit({ columnName: "a" }));
      store.stageCellEdit("tab-1", makeCellEdit({ columnName: "b" }));
      const id = changes("tab-1")![0].id;

      store.revertById("tab-1", id);
      const c = changes("tab-1");
      expect(c).toHaveLength(1);
      expect((c![0] as StagedCellEdit).columnName).toBe("b");
    });
  });

  describe("clearFailed", () => {
    it("removes error flags from all changes", () => {
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", makeCellEdit({ columnName: "a" }));
      store.stageCellEdit("tab-1", makeCellEdit({ columnName: "b" }));
      store.markFailed("tab-1", [0, 1], "fail");
      expect(changes("tab-1")![0].error).toBe("fail");
      store.clearFailed("tab-1");
      expect(changes("tab-1")![0].error).toBeUndefined();
      expect(changes("tab-1")![1].error).toBeUndefined();
    });
  });
});
