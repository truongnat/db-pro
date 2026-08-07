import { afterEach, describe, expect, it } from "vitest";

import {
  useStagedChangesStore,
  useStagedChangeCount,
  useTabStagedChanges,
} from "../state/staged-changes.store";
import type { CellValue, StagedCellEdit, StagedRowDelete } from "../state/staged-changes.store";

/* ---- helpers ---- */

const text = (v: string): CellValue => ({ type: "text", value: v });
const int = (v: number): CellValue => ({ type: "int64", value: v });

const TAB = "tab-1";

function resetStore() {
  useStagedChangesStore.getState().revertAll(TAB);
  // Also gc any leftover tabs
  useStagedChangesStore.getState().gc(new Set());
}

function makeCellEdit(overrides: Partial<StagedCellEdit> = {}): StagedCellEdit {
  return {
    kind: "cell-edit",
    pkValues: [int(1)],
    currentValues: [int(1), text("Alice")],
    columnName: "name",
    newValue: text("Bob"),
    ...overrides,
  };
}

/* ---- tests ---- */

describe("StagedChangesStore", () => {
  afterEach(resetStore);

  it("starts empty", () => {
    expect(useStagedChangesStore.getState().changes[TAB]).toBeUndefined();
  });

  it("stageCellEdit adds a change", () => {
    useStagedChangesStore.getState().stageCellEdit(TAB, makeCellEdit());
    const changes = useStagedChangesStore.getState().changes[TAB];
    expect(changes).toHaveLength(1);
    expect(changes![0].kind).toBe("cell-edit");
  });

  it("stageCellEdit replaces edit for same row + column", () => {
    const store = useStagedChangesStore.getState();
    store.stageCellEdit(TAB, makeCellEdit({ newValue: text("Bob") }));
    store.stageCellEdit(TAB, makeCellEdit({ newValue: text("Charlie") }));

    const changes = useStagedChangesStore.getState().changes[TAB];
    expect(changes).toHaveLength(1);
    expect((changes![0] as StagedCellEdit).newValue).toEqual(text("Charlie"));
  });

  it("stageCellEdit keeps edits for different columns", () => {
    const store = useStagedChangesStore.getState();
    store.stageCellEdit(TAB, makeCellEdit({ columnName: "name", newValue: text("Bob") }));
    store.stageCellEdit(TAB, makeCellEdit({ columnName: "email", newValue: text("bob@test.com") }));

    const changes = useStagedChangesStore.getState().changes[TAB];
    expect(changes).toHaveLength(2);
  });

  it("stageCellEdit keeps edits for different rows", () => {
    const store = useStagedChangesStore.getState();
    store.stageCellEdit(TAB, makeCellEdit({ pkValues: [int(1)] }));
    store.stageCellEdit(TAB, makeCellEdit({ pkValues: [int(2)] }));

    const changes = useStagedChangesStore.getState().changes[TAB];
    expect(changes).toHaveLength(2);
  });

  it("stageDeleteRow adds a delete", () => {
    useStagedChangesStore.getState().stageDeleteRow(TAB, [int(1)]);
    const changes = useStagedChangesStore.getState().changes[TAB];
    expect(changes).toHaveLength(1);
    expect(changes![0].kind).toBe("row-delete");
  });

  it("stageDeleteRow removes cell-edits for the same row", () => {
    const store = useStagedChangesStore.getState();
    store.stageCellEdit(TAB, makeCellEdit({ pkValues: [int(1)] }));
    store.stageCellEdit(TAB, makeCellEdit({ pkValues: [int(2)] }));
    store.stageDeleteRow(TAB, [int(1)]);

    const changes = useStagedChangesStore.getState().changes[TAB];
    expect(changes).toHaveLength(2); // 1 remaining edit + 1 delete
    const edit = changes.find((c) => c.kind === "cell-edit");
    expect(edit).toBeDefined();
    expect((edit as StagedCellEdit).pkValues).toEqual([int(2)]);
    const del = changes.find((c) => c.kind === "row-delete");
    expect(del).toBeDefined();
  });

  it("stageDeleteRow is idempotent for same row", () => {
    const store = useStagedChangesStore.getState();
    store.stageDeleteRow(TAB, [int(1)]);
    store.stageDeleteRow(TAB, [int(1)]);

    const changes = useStagedChangesStore.getState().changes[TAB];
    expect(changes).toHaveLength(1);
  });

  it("revertAt removes a specific change", () => {
    const store = useStagedChangesStore.getState();
    store.stageCellEdit(TAB, makeCellEdit({ columnName: "name" }));
    store.stageCellEdit(TAB, makeCellEdit({ columnName: "email" }));
    store.revertAt(TAB, 0);

    const changes = useStagedChangesStore.getState().changes[TAB];
    expect(changes).toHaveLength(1);
    expect((changes![0] as StagedCellEdit).columnName).toBe("email");
  });

  it("revertAt is no-op for out-of-range index", () => {
    const store = useStagedChangesStore.getState();
    store.stageCellEdit(TAB, makeCellEdit());
    store.revertAt(TAB, 99);
    store.revertAt(TAB, -1);

    const changes = useStagedChangesStore.getState().changes[TAB];
    expect(changes).toHaveLength(1);
  });

  it("revertAll clears all changes for a tab", () => {
    const store = useStagedChangesStore.getState();
    store.stageCellEdit(TAB, makeCellEdit());
    store.stageDeleteRow(TAB, [int(2)]);
    store.revertAll(TAB);

    expect(useStagedChangesStore.getState().changes[TAB]).toBeUndefined();
  });

  it("clearChanges clears changes for a tab", () => {
    const store = useStagedChangesStore.getState();
    store.stageCellEdit(TAB, makeCellEdit());
    store.clearChanges(TAB);

    expect(useStagedChangesStore.getState().changes[TAB]).toBeUndefined();
  });

  it("gc removes changes for closed tabs", () => {
    const store = useStagedChangesStore.getState();
    store.stageCellEdit("tab-1", makeCellEdit());
    store.stageCellEdit("tab-2", makeCellEdit());
    store.stageCellEdit("tab-3", makeCellEdit());

    store.gc(new Set(["tab-1", "tab-3"]));

    const state = useStagedChangesStore.getState();
    expect(state.changes["tab-1"]).toHaveLength(1);
    expect(state.changes["tab-2"]).toBeUndefined();
    expect(state.changes["tab-3"]).toHaveLength(1);
  });

  it("isolates changes between tabs", () => {
    const store = useStagedChangesStore.getState();
    store.stageCellEdit("tab-a", makeCellEdit({ newValue: text("A") }));
    store.stageCellEdit("tab-b", makeCellEdit({ newValue: text("B") }));

    expect(useStagedChangesStore.getState().changes["tab-a"]).toHaveLength(1);
    expect(useStagedChangesStore.getState().changes["tab-b"]).toHaveLength(1);
    expect(
      (useStagedChangesStore.getState().changes["tab-a"]![0] as StagedCellEdit).newValue,
    ).toEqual(text("A"));
  });
});
