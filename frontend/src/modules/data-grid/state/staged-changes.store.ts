import { create } from "zustand";

import type { CellValue } from "@/modules/query/types/query.types";

/* ------------------------------------------------------------------ */
/*  Types                                                              */
/* ------------------------------------------------------------------ */

/** A single cell modification staged for review / apply. */
export interface StagedCellEdit {
  kind: "cell-edit";
  /** Primary-key column values that identify the row. */
  pkValues: CellValue[];
  /** All column values *after* the edit (full row snapshot). */
  currentValues: CellValue[];
  /** Column name that was changed. */
  columnName: string;
  /** The new value the user typed. */
  newValue: CellValue;
}

/** A row deletion staged for review / apply. */
export interface StagedRowDelete {
  kind: "row-delete";
  pkValues: CellValue[];
}

export type StagedChange = (StagedCellEdit | StagedRowDelete) & { error?: string };

/* ------------------------------------------------------------------ */
/*  Store shape                                                        */
/* ------------------------------------------------------------------ */

interface StagedChangesState {
  /** Per-tab list of staged changes. */
  changes: Record<string, StagedChange[]>;
}

interface StagedChangesActions {
  /** Stage a cell edit. Replaces any previous edit for the same row+column. */
  stageCellEdit: (tabId: string, edit: StagedCellEdit) => void;
  /** Stage a row deletion. */
  stageDeleteRow: (tabId: string, pkValues: CellValue[]) => void;
  /** Revert a single staged change by index. */
  revertAt: (tabId: string, index: number) => void;
  /** Revert all staged changes for a tab. */
  revertAll: (tabId: string) => void;
  /** Clear staged changes after successful apply. */
  clearChanges: (tabId: string) => void;
  /** Mark specific change indices as failed with an error message. */
  markFailed: (tabId: string, indices: number[], error: string) => void;
  /** Clear error flags from all changes for a tab. */
  clearFailed: (tabId: string) => void;
  /** Keep only changes at given indices (remove successful ones after partial apply). */
  keepOnlyIndices: (tabId: string, indices: number[]) => void;
  /** Garbage-collect tabs no longer open. */
  gc: (validTabIds: Set<string>) => void;
}

export type StagedChangesStore = StagedChangesState & StagedChangesActions;

/* ------------------------------------------------------------------ */
/*  Helpers                                                            */
/* ------------------------------------------------------------------ */

function pkKey(pkValues: CellValue[]): string {
  return pkValues.map((v) => JSON.stringify(v)).join("|");
}

/* ------------------------------------------------------------------ */
/*  Store                                                              */
/* ------------------------------------------------------------------ */

export const useStagedChangesStore = create<StagedChangesStore>()((set, _get) => ({
  changes: {},

  stageCellEdit: (tabId, edit) =>
    set((s) => {
      const existing = s.changes[tabId] ?? [];
      // Replace any previous cell-edit for the same row + column
      const filtered = existing.filter((c) => {
        if (c.kind !== "cell-edit") return true;
        const sameRow = pkKey(c.pkValues) === pkKey(edit.pkValues);
        return !(sameRow && c.columnName === edit.columnName);
      });
      return { changes: { ...s.changes, [tabId]: [...filtered, edit] } };
    }),

  stageDeleteRow: (tabId, pkValues) =>
    set((s) => {
      const existing = s.changes[tabId] ?? [];
      // Remove any cell-edits for this row (they're subsumed by the delete)
      const key = pkKey(pkValues);
      const filtered = existing.filter((c) => {
        if (c.kind === "cell-edit") return pkKey(c.pkValues) !== key;
        if (c.kind === "row-delete") return pkKey(c.pkValues) !== key;
        return true;
      });
      return {
        changes: { ...s.changes, [tabId]: [...filtered, { kind: "row-delete", pkValues }] },
      };
    }),

  revertAt: (tabId, index) =>
    set((s) => {
      const existing = s.changes[tabId] ?? [];
      if (index < 0 || index >= existing.length) return s;
      const next = [...existing];
      next.splice(index, 1);
      return { changes: { ...s.changes, [tabId]: next } };
    }),

  revertAll: (tabId) =>
    set((s) => {
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      const { [tabId]: _, ...rest } = s.changes;
      return { changes: rest };
    }),

  clearChanges: (tabId) =>
    set((s) => {
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      const { [tabId]: __, ...rest } = s.changes;
      return { changes: rest };
    }),

  markFailed: (tabId, indices, error) =>
    set((s) => {
      const existing = s.changes[tabId] ?? [];
      const failSet = new Set(indices);
      const updated = existing.map((c, i) => (failSet.has(i) ? { ...c, error } : c));
      return { changes: { ...s.changes, [tabId]: updated } };
    }),

  clearFailed: (tabId) =>
    set((s) => {
      const existing = s.changes[tabId] ?? [];
      const updated = existing.map((c) => {
        if (c.error) {
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          const { error: _, ...rest } = c;
          return rest as typeof c;
        }
        return c;
      });
      return { changes: { ...s.changes, [tabId]: updated } };
    }),

  keepOnlyIndices: (tabId, indices) =>
    set((s) => {
      const existing = s.changes[tabId] ?? [];
      const keepSet = new Set(indices);
      const kept = existing.filter((_, i) => keepSet.has(i));
      return { changes: { ...s.changes, [tabId]: kept } };
    }),

  gc: (validTabIds) =>
    set((s) => {
      const cleaned: Record<string, StagedChange[]> = {};
      for (const [id, changes] of Object.entries(s.changes)) {
        if (validTabIds.has(id)) cleaned[id] = changes;
      }
      return { changes: cleaned };
    }),
}));

/* ------------------------------------------------------------------ */
/*  Selectors                                                          */
/* ------------------------------------------------------------------ */

/** Get the staged changes for a specific tab. */
export function useTabStagedChanges(tabId: string): StagedChange[] {
  return useStagedChangesStore((s) => s.changes[tabId] ?? []);
}

/** Get just the count of staged changes for a tab. */
export function useStagedChangeCount(tabId: string): number {
  return useStagedChangesStore((s) => (s.changes[tabId] ?? []).length);
}
