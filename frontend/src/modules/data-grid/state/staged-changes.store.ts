import type { CellValue } from "@/modules/query/types/query.types";
import { create } from "zustand";
import { persist } from "zustand/middleware";

// ─── Types ────────────────────────────────────────────────────────────────────

/**
 * A patch-style row update. `changes` is a map of column name → new value.
 * Only modified columns are included; untouched columns are absent.
 */
export interface StagedCellEdit {
  id: string;
  kind: "cell-edit";
  pkValues: CellValue[];
  changes: Record<string, CellValue>;
  /** Non-null when the most recent apply attempt for this revision failed. */
  error: string | null;
}

export interface StagedRowInsert {
  id: string;
  kind: "row-insert";
  values: CellValue[];
}

export interface StagedRowDelete {
  id: string;
  kind: "row-delete";
  pkValues: CellValue[];
  /** Non-null when the most recent apply attempt for this revision failed. */
  error: string | null;
}

export type StagedChange = StagedCellEdit | StagedRowInsert | StagedRowDelete;

export type StagedChangeKind = StagedChange["kind"];

/**
 * Filter predicate for staged changes. Accepts either a kind string
 * or a predicate function for more complex filtering.
 */
export type StagedChangeFilter =
  | StagedChangeKind
  | ((change: StagedChange) => boolean);

/**
 * Convert a StagedChangeFilter to a predicate function.
 */
function toPredicate(filter?: StagedChangeFilter): (change: StagedChange) => boolean {
  if (!filter) return () => true;
  if (typeof filter === "function") return filter;
  return (change: StagedChange) => change.kind === filter;
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

let idCounter = 0;

/** Generate a stable, unique ID for each staged revision. */
export function generateChangeId(): string {
  return `sc-${Date.now().toString(36)}-${(++idCounter).toString(36)}`;
}

/** Create a canonical string key from primary key values for comparison. */
export function pkKey(pkValues: CellValue[]): string {
  return pkValues
    .map((v) => {
      if (v.type === "null") return "NULL";
      return `${v.type}:${String(v.value)}`;
    })
    .join("|");
}

// ─── Store State ──────────────────────────────────────────────────────────────

export interface StagedChangesState {
  /**
   * Map of tabId → array of staged changes.
   * Each entry is an immutable revision: once created, its ID never changes.
   * When the user edits an in-flight revision, a NEW revision is created.
   */
  changes: Record<string, StagedChange[]>;

  /** IDs currently being sent to the backend by applyChanges. */
  inFlightIds: Set<string>;

  // ─── Actions ──────────────────────────────────────────────────────────────

  /**
   * Stage a cell edit as a patch. If a non-in-flight entry exists for the
   * same PK, the patch is merged into it (same ID). If the existing entry
   * is in-flight, a NEW revision is created with a new ID.
   */
  stageCellEdit: (tabId: string, edit: Omit<StagedCellEdit, "id" | "kind" | "error">) => void;

  /** Stage a new row insert. */
  stageRowInsert: (tabId: string, insert: Omit<StagedRowInsert, "id" | "kind">) => void;

  /**
   * Stage a row delete. Removes any existing cell-edit for the same PK
   * (since the delete supersedes them). Creates a new revision.
   */
  stageDeleteRow: (tabId: string, deleteOp: Omit<StagedRowDelete, "id" | "kind" | "error">) => void;

  /**
   * Mark specific staged revisions as in-flight (being sent to backend).
   * While in-flight, edits to the same row create new revisions.
   */
  markInFlight: (tabId: string, ids: string[]) => void;

  /** Remove specific staged revisions by ID (successful apply cleanup). */
  removeByIds: (tabId: string, ids: string[]) => void;

  /** Mark specific revisions as failed with error messages. */
  markFailedByIds: (tabId: string, failures: Array<{ id: string; error: string }>) => void;

  /** Revert a specific staged revision by ID. */
  revertById: (tabId: string, id: string) => void;

  /** Clear all failed states for a tab. */
  clearFailed: (tabId: string) => void;

  /** Clear all staged revisions for a tab. */
  clearTab: (tabId: string) => void;

  /** Clear all staged revisions across all tabs. */
  clearAll: () => void;

  // ─── Selectors ────────────────────────────────────────────────────────────

  /** Get all staged revisions for a tab. */
  getChanges: (tabId: string) => StagedChange[];

  /** Get staged revisions for a tab, optionally filtered. */
  getFilteredChanges: (tabId: string, filter?: StagedChangeFilter) => StagedChange[];

  /** Get count of staged revisions for a tab. */
  getCount: (tabId: string) => number;

  /** Get count of staged revisions for a tab, optionally filtered. */
  getFilteredCount: (tabId: string, filter?: StagedChangeFilter) => number;

  /** Check if a specific revision has an error. */
  hasError: (tabId: string, id: string) => boolean;

  /** Check if any revision for a tab has an error, optionally filtered by kind. */
  hasErrors: (tabId: string, filter?: StagedChangeKind) => boolean;

  /** Get all revision IDs that have errors for a tab. */
  getErrorIds: (tabId: string, filter?: StagedChangeKind) => string[];
}

// ─── Store ────────────────────────────────────────────────────────────────────

export const useStagedChangesStore = create<StagedChangesState>()(
  persist(
    (set, get) => ({
      changes: {},
      inFlightIds: new Set<string>(),

      stageCellEdit: (tabId, edit) =>
        set((s) => {
          const existing = s.changes[tabId] ?? [];
          const key = pkKey(edit.pkValues);

          // Find existing cell-edit for same PK
          const editIdx = existing.findIndex(
            (c) => c.kind === "cell-edit" && pkKey(c.pkValues) === key,
          );

          if (editIdx !== -1) {
            const prev = existing[editIdx] as StagedCellEdit;
            if (s.inFlightIds.has(prev.id)) {
              // In-flight: create NEW revision (caller composed the patch)
              return {
                changes: {
                  ...s.changes,
                  [tabId]: [...existing, { ...edit, kind: "cell-edit" as const, id: generateChangeId(), error: null }],
                },
              };
            }
            // Not in-flight: merge into existing revision (same ID)
            const merged: StagedCellEdit = {
              ...prev,
              changes: { ...edit.changes },
            };
            const next = [...existing];
            next[editIdx] = merged;
            return { changes: { ...s.changes, [tabId]: next } };
          }

          // No existing entry — create new revision
          return {
            changes: {
              ...s.changes,
              [tabId]: [...existing, { ...edit, kind: "cell-edit" as const, id: generateChangeId(), error: null }],
            },
          };
        }),

      stageRowInsert: (tabId, insert) =>
        set((s) => {
          const existing = s.changes[tabId] ?? [];
          return {
            changes: {
              ...s.changes,
              [tabId]: [...existing, { ...insert, kind: "row-insert" as const, id: generateChangeId() }],
            },
          };
        }),

      stageDeleteRow: (tabId, deleteOp) =>
        set((s) => {
          const existing = s.changes[tabId] ?? [];
          const key = pkKey(deleteOp.pkValues);

          // Remove any cell-edit or delete for same PK (delete supersedes)
          const filtered = existing.filter(
            (c) =>
              !(
                (c.kind === "cell-edit" || c.kind === "row-delete") &&
                pkKey(c.pkValues) === key
              ),
          );

          return {
            changes: {
              ...s.changes,
              [tabId]: [...filtered, { ...deleteOp, kind: "row-delete" as const, id: generateChangeId(), error: null }],
            },
          };
        }),

      markInFlight: (tabId, ids) =>
        set((s) => {
          const next = new Set(s.inFlightIds);
          for (const id of ids) next.add(id);
          return { inFlightIds: next };
        }),

      removeByIds: (tabId, ids) =>
        set((s) => {
          const idSet = new Set(ids);
          const existing = s.changes[tabId] ?? [];
          const nextInFlight = new Set(s.inFlightIds);
          for (const id of ids) nextInFlight.delete(id);
          return {
            changes: {
              ...s.changes,
              [tabId]: existing.filter((c) => !idSet.has(c.id)),
            },
            inFlightIds: nextInFlight,
          };
        }),

      markFailedByIds: (tabId, failures) =>
        set((s) => {
          const failureMap = new Map(failures.map((f) => [f.id, f.error]));
          const existing = s.changes[tabId] ?? [];
          const nextInFlight = new Set(s.inFlightIds);
          for (const id of failureMap.keys()) nextInFlight.delete(id);
          return {
            changes: {
              ...s.changes,
              [tabId]: existing.map((c) =>
                failureMap.has(c.id) ? { ...c, error: failureMap.get(c.id)! } : c,
              ),
            },
            inFlightIds: nextInFlight,
          };
        }),

      revertById: (tabId, id) =>
        set((s) => {
          const existing = s.changes[tabId] ?? [];
          return {
            changes: {
              ...s.changes,
              [tabId]: existing.filter((c) => c.id !== id),
            },
          };
        }),

      clearFailed: (tabId) =>
        set((s) => {
          const existing = s.changes[tabId] ?? [];
          return {
            changes: {
              ...s.changes,
              [tabId]: existing.map((c) =>
                c.kind !== "row-insert" ? { ...c, error: null } : c,
              ),
            },
          };
        }),

      clearTab: (tabId) =>
        set((s) => {
          const next: Record<string, StagedChange[]> = { ...s.changes };
          delete next[tabId];
          return { changes: next };
        }),

      clearAll: () => set({ changes: {} }),

      getChanges: (tabId) => get().changes[tabId] ?? [],

      getFilteredChanges: (tabId, filter) => {
        const predicate = toPredicate(filter);
        return get().changes[tabId]?.filter(predicate) ?? [];
      },

      getCount: (tabId) => get().changes[tabId]?.length ?? 0,

      getFilteredCount: (tabId, filter) => {
        const predicate = toPredicate(filter);
        const changes = get().changes[tabId] ?? [];
        return changes.filter(predicate).length;
      },

      hasError: (tabId, id) => {
        const changes = get().changes[tabId] ?? [];
        return changes.some((c) => c.id === id && c.kind !== "row-insert" && c.error !== null);
      },

      hasErrors: (tabId, filter) => {
        const changes = get().changes[tabId] ?? [];
        return changes.some((c) => {
          if (c.kind === "row-insert") return false;
          if (filter && c.kind !== filter) return false;
          return c.error !== null;
        });
      },

      getErrorIds: (tabId, filter) => {
        const changes = get().changes[tabId] ?? [];
        return changes
          .filter((c) => {
            if (c.kind === "row-insert") return false;
            if (filter && c.kind !== filter) return false;
            return c.error !== null;
          })
          .map((c) => c.id);
      },
    }),
    {
      name: "db-pro-staged-changes",
      partialize: (state) => ({
        changes: state.changes,
      }),
    },
  ),
);

/** Reactive selector: staged changes for a specific tab. */
export function useTabStagedChanges(tabId: string): StagedChange[] {
  return useStagedChangesStore((s) => s.changes[tabId] ?? []);
}
