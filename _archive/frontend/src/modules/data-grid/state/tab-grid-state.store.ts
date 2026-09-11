import { create } from "zustand";
import { persist } from "zustand/middleware";

import type { GridFilter, GridSort } from "../types/data-grid.types";
import type { ChartConfig } from "./data-grid.store";

export interface GridTabState {
  filters: GridFilter[];
  sorts: GridSort[];
  draftFilters: GridFilter[];
  draftSorts: GridSort[];
  page: number;
  pageSize: number;
  editingCell: { row: number; col: number } | null;
  frozenColumns: string[];
  hiddenColumns: string[];
  columnWidths: Record<string, number>;
  chartConfig: ChartConfig | null;
}

const defaultGridState: GridTabState = {
  filters: [],
  sorts: [],
  draftFilters: [],
  draftSorts: [],
  page: 1,
  pageSize: 50,
  editingCell: null,
  frozenColumns: [],
  hiddenColumns: [],
  columnWidths: {},
  chartConfig: null,
};

interface TabGridStateStore {
  states: Record<string, GridTabState>;
  getState: (tabId: string) => GridTabState;
  setState: (tabId: string, partial: Partial<GridTabState>) => void;
  /** Add a filter to the draft list (does NOT trigger fetch). */
  addDraftFilter: (tabId: string, filter: GridFilter) => void;
  /** Remove a filter from the draft list by index (does NOT trigger fetch). */
  removeDraftFilter: (tabId: string, index: number) => void;
  /** Set draft filters (does NOT trigger fetch). */
  setDraftFilters: (tabId: string, filters: GridFilter[]) => void;
  /** Apply draft filters → applied filters, reset page, trigger fetch. */
  applyFilters: (tabId: string) => void;
  /** Clear all filters (draft + applied), reset page. */
  clearFilters: (tabId: string) => void;
  /** Add a sort to the draft list (does NOT trigger fetch). */
  addDraftSort: (tabId: string, sort: GridSort) => void;
  /** Remove a sort from the draft list by index (does NOT trigger fetch). */
  removeDraftSort: (tabId: string, index: number) => void;
  /** Set draft sorts (does NOT trigger fetch). */
  setDraftSorts: (tabId: string, sorts: GridSort[]) => void;
  /** Apply draft sorts → applied sorts, trigger fetch. */
  applySorts: (tabId: string) => void;
  /** Clear all sorts (draft + applied). */
  clearSorts: (tabId: string) => void;
  /** Set applied sorts directly (for header-click immediate sort). */
  setSorts: (tabId: string, sorts: GridSort[]) => void;
  setPage: (tabId: string, page: number) => void;
  setPageSize: (tabId: string, size: number) => void;
  setEditingCell: (tabId: string, cell: { row: number; col: number } | null) => void;
  toggleFrozenColumn: (tabId: string, column: string) => void;
  toggleHiddenColumn: (tabId: string, column: string) => void;
  setHiddenColumns: (tabId: string, columns: string[]) => void;
  setColumnWidths: (tabId: string, widths: Record<string, number>) => void;
  setChartConfig: (tabId: string, config: ChartConfig | null) => void;
  resetTab: (tabId: string) => void;
  gc: (validTabIds: Set<string>) => void;
}

function ensureTab(states: Record<string, GridTabState>, tabId: string): GridTabState {
  return states[tabId] ?? { ...defaultGridState };
}

export const useTabGridStateStore = create<TabGridStateStore>()(
  persist(
    (set, get) => ({
      states: {},

      getState: (tabId) => ensureTab(get().states, tabId),

      setState: (tabId, partial) =>
        set((s) => ({
          states: {
            ...s.states,
            [tabId]: { ...ensureTab(s.states, tabId), ...partial },
          },
        })),

      addDraftFilter: (tabId, filter) =>
        set((s) => {
          const current = ensureTab(s.states, tabId);
          return {
            states: {
              ...s.states,
              [tabId]: { ...current, draftFilters: [...current.draftFilters, filter] },
            },
          };
        }),

      removeDraftFilter: (tabId, index) =>
        set((s) => {
          const current = ensureTab(s.states, tabId);
          return {
            states: {
              ...s.states,
              [tabId]: {
                ...current,
                draftFilters: current.draftFilters.filter((_, i) => i !== index),
              },
            },
          };
        }),

      setDraftFilters: (tabId, filters) =>
        set((s) => {
          const current = ensureTab(s.states, tabId);
          return { states: { ...s.states, [tabId]: { ...current, draftFilters: filters } } };
        }),

      applyFilters: (tabId) =>
        set((s) => {
          const current = ensureTab(s.states, tabId);
          return {
            states: {
              ...s.states,
              [tabId]: { ...current, filters: [...current.draftFilters], page: 1 },
            },
          };
        }),

      clearFilters: (tabId) =>
        set((s) => {
          const current = ensureTab(s.states, tabId);
          return {
            states: {
              ...s.states,
              [tabId]: { ...current, filters: [], draftFilters: [], page: 1 },
            },
          };
        }),

      addDraftSort: (tabId, sort) =>
        set((s) => {
          const current = ensureTab(s.states, tabId);
          const next = [...current.draftSorts.filter((x) => x.column !== sort.column), sort];
          return { states: { ...s.states, [tabId]: { ...current, draftSorts: next } } };
        }),

      removeDraftSort: (tabId, index) =>
        set((s) => {
          const current = ensureTab(s.states, tabId);
          return {
            states: {
              ...s.states,
              [tabId]: {
                ...current,
                draftSorts: current.draftSorts.filter((_, i) => i !== index),
              },
            },
          };
        }),

      setDraftSorts: (tabId, sorts) =>
        set((s) => {
          const current = ensureTab(s.states, tabId);
          return { states: { ...s.states, [tabId]: { ...current, draftSorts: sorts } } };
        }),

      applySorts: (tabId) =>
        set((s) => {
          const current = ensureTab(s.states, tabId);
          return {
            states: {
              ...s.states,
              [tabId]: { ...current, sorts: [...current.draftSorts], page: 1 },
            },
          };
        }),

      clearSorts: (tabId) =>
        set((s) => {
          const current = ensureTab(s.states, tabId);
          return {
            states: {
              ...s.states,
              [tabId]: { ...current, sorts: [], draftSorts: [], page: 1 },
            },
          };
        }),

      setSorts: (tabId, sorts) =>
        set((s) => ({
          states: {
            ...s.states,
            [tabId]: { ...ensureTab(s.states, tabId), sorts, page: 1 },
          },
        })),

      setPage: (tabId, page) =>
        set((s) => ({
          states: {
            ...s.states,
            [tabId]: { ...ensureTab(s.states, tabId), page },
          },
        })),

      setPageSize: (tabId, size) =>
        set((s) => ({
          states: {
            ...s.states,
            [tabId]: { ...ensureTab(s.states, tabId), pageSize: size, page: 1 },
          },
        })),

      setEditingCell: (tabId, cell) =>
        set((s) => ({
          states: {
            ...s.states,
            [tabId]: { ...ensureTab(s.states, tabId), editingCell: cell },
          },
        })),

      toggleFrozenColumn: (tabId, column) =>
        set((s) => {
          const current = ensureTab(s.states, tabId);
          const frozen = current.frozenColumns.includes(column)
            ? current.frozenColumns.filter((c) => c !== column)
            : [...current.frozenColumns, column];
          return {
            states: { ...s.states, [tabId]: { ...current, frozenColumns: frozen } },
          };
        }),

      toggleHiddenColumn: (tabId, column) =>
        set((s) => {
          const current = ensureTab(s.states, tabId);
          const hidden = current.hiddenColumns.includes(column)
            ? current.hiddenColumns.filter((c) => c !== column)
            : [...current.hiddenColumns, column];
          return {
            states: { ...s.states, [tabId]: { ...current, hiddenColumns: hidden } },
          };
        }),

      setHiddenColumns: (tabId, columns) =>
        set((s) => {
          const current = ensureTab(s.states, tabId);
          return {
            states: { ...s.states, [tabId]: { ...current, hiddenColumns: columns } },
          };
        }),

      setColumnWidths: (tabId, widths) =>
        set((s) => {
          const current = ensureTab(s.states, tabId);
          return {
            states: { ...s.states, [tabId]: { ...current, columnWidths: widths } },
          };
        }),

      setChartConfig: (tabId, config) =>
        set((s) => ({
          states: {
            ...s.states,
            [tabId]: { ...ensureTab(s.states, tabId), chartConfig: config },
          },
        })),

      resetTab: (tabId) =>
        set((s) => {
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          const { [tabId]: _, ...rest } = s.states;
          return { states: rest };
        }),

      gc: (validTabIds) =>
        set((s) => {
          const cleaned: Record<string, GridTabState> = {};
          for (const [id, state] of Object.entries(s.states)) {
            if (validTabIds.has(id)) cleaned[id] = state;
          }
          return { states: cleaned };
        }),
    }),
    {
      name: "dbpro:grid",
      partialize: (s) => ({ states: s.states }),
    },
  ),
);
