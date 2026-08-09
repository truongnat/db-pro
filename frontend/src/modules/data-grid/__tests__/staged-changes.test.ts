import { afterEach, describe, expect, it } from "vitest";

import {
  useStagedChangesStore,
} from "../state/staged-changes.store";
import type { StagedCellEdit } from "../state/staged-changes.store";
import type { CellValue } from "@/modules/query/types/query.types";

/* ---- helpers ---- */

const text = (v: string): CellValue => ({ type: "text", value: v });
const int = (v: number): CellValue => ({ type: "int64", value: v });

const TAB = "tab-1";

function resetStore() {
  useStagedChangesStore.getState().clearTab(TAB);
}

function makeCellEdit(overrides: { pkValues?: CellValue[]; changes?: Record<string, CellValue> } = {}) {
  return {
    pkValues: overrides.pkValues ?? [int(1)],
    changes: overrides.changes ?? { name: text("Bob") },
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
    store.stageCellEdit(TAB, makeCellEdit({ changes: { name: text("Bob") } }));
    store.stageCellEdit(TAB, makeCellEdit({ changes: { name: text("Charlie") } }));

    const changes = useStagedChangesStore.getState().changes[TAB];
    expect(changes).toHaveLength(1);
    expect((changes![0] as StagedCellEdit).changes.name).toEqual(text("Charlie"));
  });

  it("stageCellEdit merges patches for different columns of same row", () => {
    const store = useStagedChangesStore.getState();
    store.stageCellEdit(TAB, makeCellEdit({ changes: { name: text("Bob") } }));
    store.stageCellEdit(TAB, makeCellEdit({ changes: { email: text("bob@test.com") } }));

    const changes = useStagedChangesStore.getState().changes[TAB];
    expect(changes).toHaveLength(1);
    const edit = changes![0] as StagedCellEdit;
    expect(edit.changes.name).toEqual(text("Bob"));
    expect(edit.changes.email).toEqual(text("bob@test.com"));
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

  it("revertById removes a specific change", () => {
    const store = useStagedChangesStore.getState();
    store.stageCellEdit(TAB, makeCellEdit({ pkValues: [int(1)], changes: { name: text("Bob") } }));
    store.stageCellEdit(TAB, makeCellEdit({ pkValues: [int(2)], changes: { email: text("x@y") } }));
    const firstId = useStagedChangesStore.getState().changes[TAB]![0].id;
    store.revertById(TAB, firstId);

    const changes = useStagedChangesStore.getState().changes[TAB];
    expect(changes).toHaveLength(1);
    expect((changes![0] as StagedCellEdit).changes.email).toEqual(text("x@y"));
  });

  it("clearTab clears all changes for a tab", () => {
    const store = useStagedChangesStore.getState();
    store.stageCellEdit(TAB, makeCellEdit());
    store.stageDeleteRow(TAB, [int(2)]);
    store.clearTab(TAB);

    expect(useStagedChangesStore.getState().changes[TAB]).toBeUndefined();
  });

  it("isolates changes between tabs", () => {
    const store = useStagedChangesStore.getState();
    store.stageCellEdit("tab-a", makeCellEdit({ changes: { name: text("A") } }));
    store.stageCellEdit("tab-b", makeCellEdit({ changes: { name: text("B") } }));

    expect(useStagedChangesStore.getState().changes["tab-a"]).toHaveLength(1);
    expect(useStagedChangesStore.getState().changes["tab-b"]).toHaveLength(1);
    expect(
      (useStagedChangesStore.getState().changes["tab-a"]![0] as StagedCellEdit).changes.name,
    ).toEqual(text("A"));
  });
});
