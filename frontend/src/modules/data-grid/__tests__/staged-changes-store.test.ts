import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { useStagedChangesStore, type StagedCellEdit } from "../state/staged-changes.store";
import type { CellValue } from "@/modules/query/types/query.types";

function resetStore() {
  useStagedChangesStore.setState({ changes: {}, inFlightIds: new Set() });
}

const int = (v: number): CellValue => ({ type: "int64", value: v });
const text = (v: string): CellValue => ({ type: "text", value: v });

/** Helper: stage a cell edit using the patch model. */
function patchEdit(pkValues: CellValue[], changes: Record<string, CellValue>) {
  return { pkValues, changes };
}

/** Helper: always read fresh state after mutations. */
function changes(tabId: string) {
  return useStagedChangesStore.getState().changes[tabId];
}

describe("StagedChangesStore — patch model", () => {
  beforeEach(resetStore);
  afterEach(resetStore);

  describe("stageCellEdit", () => {
    it("stages a cell edit as a patch", () => {
      useStagedChangesStore
        .getState()
        .stageCellEdit("tab-1", patchEdit([int(1)], { name: text("Bob") }));
      const c = changes("tab-1");
      expect(c).toHaveLength(1);
      expect(c![0].kind).toBe("cell-edit");
      expect((c![0] as StagedCellEdit).changes).toEqual({ name: text("Bob") });
    });

    it("merges patches for same column → latest value wins", () => {
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", patchEdit([int(1)], { name: text("Bob") }));
      store.stageCellEdit("tab-1", patchEdit([int(1)], { name: text("Charlie") }));
      const c = changes("tab-1");
      expect(c).toHaveLength(1);
      expect((c![0] as StagedCellEdit).changes.name).toEqual(text("Charlie"));
    });

    it("merges patches for different columns → both present", () => {
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", patchEdit([int(1)], { name: text("Bob") }));
      store.stageCellEdit("tab-1", patchEdit([int(1)], { age: int(21) }));
      const c = changes("tab-1");
      expect(c).toHaveLength(1);
      const edit = c![0] as StagedCellEdit;
      expect(edit.changes.name).toEqual(text("Bob"));
      expect(edit.changes.age).toEqual(int(21));
    });

    it("keeps edits for different rows separate", () => {
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", patchEdit([int(1)], { name: text("Bob") }));
      store.stageCellEdit("tab-1", patchEdit([int(2)], { name: text("Carol") }));
      expect(changes("tab-1")).toHaveLength(2);
    });

    it("isolates changes per tab", () => {
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", patchEdit([int(1)], { name: text("Bob") }));
      store.stageCellEdit("tab-2", patchEdit([int(1)], { email: text("x@y") }));
      expect(changes("tab-1")).toHaveLength(1);
      expect(changes("tab-2")).toHaveLength(1);
    });
  });

  describe("stageDeleteRow", () => {
    it("stages a row deletion", () => {
      useStagedChangesStore.getState().stageDeleteRow("tab-1", [int(1)]);
      const c = changes("tab-1");
      expect(c).toHaveLength(1);
      expect(c![0].kind).toBe("row-delete");
    });

    it("removes cell edits for the same row when deleting", () => {
      const pk = [int(1)];
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", patchEdit(pk, { name: text("Bob") }));
      expect(changes("tab-1")).toHaveLength(1);
      store.stageDeleteRow("tab-1", pk);
      const c = changes("tab-1");
      expect(c).toHaveLength(1);
      expect(c![0].kind).toBe("row-delete");
    });

    it("does not remove cell edits for different rows", () => {
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", patchEdit([int(1)], { name: text("Bob") }));
      store.stageDeleteRow("tab-1", [int(2)]);
      expect(changes("tab-1")).toHaveLength(2);
    });

    it("replaces duplicate delete for same row", () => {
      const store = useStagedChangesStore.getState();
      store.stageDeleteRow("tab-1", [int(1)]);
      store.stageDeleteRow("tab-1", [int(1)]);
      expect(changes("tab-1")).toHaveLength(1);
    });
  });

  describe("removeByIds (identity-safe apply)", () => {
    it("removes only changes with matching IDs", () => {
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", patchEdit([int(1)], { a: text("x") }));
      store.stageCellEdit("tab-1", patchEdit([int(2)], { b: text("y") }));
      store.stageCellEdit("tab-1", patchEdit([int(3)], { c: text("z") }));
      const allIds = changes("tab-1")!.map((c) => c.id);

      store.removeByIds("tab-1", [allIds[0], allIds[1]]);
      const c = changes("tab-1");
      expect(c).toHaveLength(1);
      expect((c![0] as StagedCellEdit).changes.c).toEqual(text("z"));
    });

    it("preserves changes staged during apply (different row)", () => {
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", patchEdit([int(1)], { a: text("x") }));
      const idA = changes("tab-1")![0].id;

      store.stageCellEdit("tab-1", patchEdit([int(2)], { b: text("y") }));

      store.removeByIds("tab-1", [idA]);
      const c = changes("tab-1");
      expect(c).toHaveLength(1);
      expect((c![0] as StagedCellEdit).changes.b).toEqual(text("y"));
    });

    it("handles unknown tab gracefully", () => {
      useStagedChangesStore.getState().removeByIds("unknown", ["x"]);
    });
  });

  describe("markFailedByIds", () => {
    it("marks specific changes by ID with error", () => {
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", patchEdit([int(1)], { a: text("x") }));
      store.stageCellEdit("tab-1", patchEdit([int(2)], { b: text("y") }));
      const ids = changes("tab-1")!.map((c) => c.id);

      store.markFailedByIds("tab-1", [{ id: ids[1], error: "DB error" }]);
      const c = changes("tab-1");
      expect(c![0].error).toBeNull();
      expect(c![1].error).toBe("DB error");
    });

    it("ignores unknown IDs", () => {
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", patchEdit([int(1)], { a: text("x") }));
      store.markFailedByIds("tab-1", [{ id: "nonexistent", error: "x" }]);
      expect(changes("tab-1")![0].error).toBeNull();
    });
  });

  describe("revertById", () => {
    it("removes change by ID", () => {
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", patchEdit([int(1)], { a: text("x") }));
      store.stageCellEdit("tab-1", patchEdit([int(2)], { b: text("y") }));
      const id = changes("tab-1")![0].id;

      store.revertById("tab-1", id);
      const c = changes("tab-1");
      expect(c).toHaveLength(1);
      expect((c![0] as StagedCellEdit).changes.b).toEqual(text("y"));
    });
  });

  describe("clearFailed", () => {
    it("removes error flags from all changes", () => {
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", patchEdit([int(1)], { a: text("x") }));
      store.stageCellEdit("tab-1", patchEdit([int(2)], { b: text("y") }));
      const ids = changes("tab-1")!.map((c) => c.id);
      store.markFailedByIds(
        "tab-1",
        ids.map((id) => ({ id, error: "fail" })),
      );
      expect(changes("tab-1")![0].error).toBe("fail");
      store.clearFailed("tab-1");
      expect(changes("tab-1")![0].error).toBeNull();
      expect(changes("tab-1")![1].error).toBeNull();
    });
  });

  describe("clearTab", () => {
    it("clears all changes for a tab", () => {
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", patchEdit([int(1)], { a: text("x") }));
      store.clearTab("tab-1");
      expect(changes("tab-1")).toBeUndefined();
    });

    it("does not affect other tabs", () => {
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", patchEdit([int(1)], { a: text("x") }));
      store.stageCellEdit("tab-2", patchEdit([int(1)], { b: text("y") }));
      store.clearTab("tab-1");
      expect(changes("tab-2")).toHaveLength(1);
    });
  });

  // ── Required regression tests (user spec A–E) ───────────────────────────

  describe("multi-cell composition (data correctness)", () => {
    const pk1 = [int(1)];

    it("A. composes edits to different columns — both values preserved", () => {
      const store = useStagedChangesStore.getState();
      // Base: name=Alice, age=20
      // Edit name → Bob
      store.stageCellEdit("tab-1", patchEdit(pk1, { name: text("Bob") }));
      // Edit age → 21
      store.stageCellEdit("tab-1", patchEdit(pk1, { age: int(21) }));

      const c = changes("tab-1");
      expect(c).toHaveLength(1);
      const edit = c![0] as StagedCellEdit;
      // BOTH edits must be present in the patch
      expect(edit.changes.name).toEqual(text("Bob"));
      expect(edit.changes.age).toEqual(int(21));
    });

    it("B. re-editing same cell — latest value wins", () => {
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", patchEdit(pk1, { name: text("Bob") }));
      store.stageCellEdit("tab-1", patchEdit(pk1, { name: text("Charlie") }));

      const c = changes("tab-1");
      expect(c).toHaveLength(1);
      expect((c![0] as StagedCellEdit).changes.name).toEqual(text("Charlie"));
    });

    it("C. edit then delete same row → only delete remains", () => {
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", patchEdit(pk1, { name: text("Bob") }));
      store.stageDeleteRow("tab-1", pk1);

      const c = changes("tab-1");
      expect(c).toHaveLength(1);
      expect(c![0].kind).toBe("row-delete");
    });

    it("D. in-flight apply + new edit → new revision survives", () => {
      const store = useStagedChangesStore.getState();
      // Stage edit A: name → Bob
      store.stageCellEdit("tab-1", patchEdit(pk1, { name: text("Bob") }));
      const idA = changes("tab-1")![0].id;

      // Mark A as in-flight (apply started)
      store.markInFlight("tab-1", [idA]);

      // User edits age → 21 while A is in-flight
      // Caller composes: { name: Bob, age: 21 }
      store.stageCellEdit("tab-1", patchEdit(pk1, { name: text("Bob"), age: int(21) }));

      // Must be TWO entries: sc-1 (in-flight) and sc-2 (new revision)
      const c = changes("tab-1");
      expect(c).toHaveLength(2);
      const idB = c![1].id;
      expect(idB).not.toBe(idA);

      // A succeeds → removeByIds only removes A
      store.removeByIds("tab-1", [idA]);

      const remaining = changes("tab-1");
      expect(remaining).toHaveLength(1);
      expect(remaining![0].id).toBe(idB);
      // B's patch must contain BOTH name and age
      const editB = remaining![0] as StagedCellEdit;
      expect(editB.changes.name).toEqual(text("Bob"));
      expect(editB.changes.age).toEqual(int(21));
    });

    it("E. partial failure marks only the immutable revision", () => {
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", patchEdit([int(1)], { a: text("x") }));
      store.stageCellEdit("tab-1", patchEdit([int(2)], { b: text("y") }));
      const ids = changes("tab-1")!.map((c) => c.id);

      store.markInFlight("tab-1", ids);

      // First succeeds, second fails
      store.removeByIds("tab-1", [ids[0]]);
      store.markFailedByIds("tab-1", [{ id: ids[1], error: "DB timeout" }]);

      const c = changes("tab-1");
      expect(c).toHaveLength(1);
      expect(c![0].id).toBe(ids[1]);
      expect(c![0].error).toBe("DB timeout");
    });
  });

  describe("revision identity (in-flight safety)", () => {
    it("merge preserves ID when NOT in-flight", () => {
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", patchEdit([int(1)], { a: text("x") }));
      const originalId = changes("tab-1")![0].id;

      store.stageCellEdit("tab-1", patchEdit([int(1)], { b: text("y") }));

      expect(changes("tab-1")).toHaveLength(1);
      expect(changes("tab-1")![0].id).toBe(originalId);
    });

    it("creates new ID when existing revision is in-flight", () => {
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", patchEdit([int(1)], { a: text("x") }));
      const idA = changes("tab-1")![0].id;

      store.markInFlight("tab-1", [idA]);
      store.stageCellEdit("tab-1", patchEdit([int(1)], { a: text("x"), b: text("y") }));

      const c = changes("tab-1");
      expect(c).toHaveLength(2);
      expect(c![0].id).toBe(idA);
      expect(c![1].id).not.toBe(idA);
    });

    it("removeByIds clears inFlightIds", () => {
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", patchEdit([int(1)], { a: text("x") }));
      const id = changes("tab-1")![0].id;

      store.markInFlight("tab-1", [id]);
      expect(useStagedChangesStore.getState().inFlightIds.has(id)).toBe(true);

      store.removeByIds("tab-1", [id]);
      expect(useStagedChangesStore.getState().inFlightIds.has(id)).toBe(false);
    });

    it("markFailedByIds clears inFlightIds", () => {
      const store = useStagedChangesStore.getState();
      store.stageCellEdit("tab-1", patchEdit([int(1)], { a: text("x") }));
      const id = changes("tab-1")![0].id;

      store.markInFlight("tab-1", [id]);
      store.markFailedByIds("tab-1", [{ id, error: "fail" }]);
      expect(useStagedChangesStore.getState().inFlightIds.has(id)).toBe(false);
    });
  });
});
