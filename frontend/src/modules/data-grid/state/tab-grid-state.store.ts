import { create } from "zustand";

import type { GridFilter, GridSort } from "../types/data-grid.types";
import type { ChartConfig } from "./data-grid.store";

export interface GridTabState {
  filters: GridFilter[];
  sorts: GridSort[];
  page: number;
  pageSize: number;
  editingCell: { row: number; col: number } | null;
  frozenColumns: string[];
  chartConfig: ChartConfig | null;
}

const defaultGridState: GridTabState = {
  filters: [],
  sorts: [],
  page: 1,
  pageSize: 50,
  editingCell: null,
  frozenColumns: [],
  chartConfig: null,
};

interface TabGridStateStore {
  states: Record<string, GridTabState>;
  getState: (tabId: string) => GridTabState;
  setState: (tabId: string, partial: Partial<GridTabState>) => void;
  addFilter: (tabId: string, filter: GridFilter) => void;
  removeFilter: (tabId: string, index: number) => void;
  setSorts: (tabId: string, sorts: GridSort[]) => void;
  setPage: (tabId: string, page: number) => void;
  setPageSize: (tabId: string, size: number) => void;
  setEditingCell: (tabId: string, cell: { row: number; col: number } | null) => void;
  toggleFrozenColumn: (tabId: string, column: string) => void;
  setChartConfig: (tabId: string, config: ChartConfig | null) => void;
  resetTab: (tabId: string) => void;
}

function ensureTab(states: Record<string, GridTabState>, tabId: string): GridTabState {
  return states[tabId] ?? { ...defaultGridState };
}

export const useTabGridStateStore = create<TabGridStateStore>()((set, get) => ({
  states: {},

  getState: (tabId) => ensureTab(get().states, tabId),

  setState: (tabId, partial) =>
    set((s) => ({
      states: {
        ...s.states,
        [tabId]: { ...ensureTab(s.states, tabId), ...partial },
      },
    })),

  addFilter: (tabId, filter) =>
    set((s) => ({
      states: {
        ...s.states,
        [tabId]: { ...ensureTab(s.states, tabId), filters: [...ensureTab(s.states, tabId).filters, filter], page: 1 },
      },
    })),

  removeFilter: (tabId, index) =>
    set((s) => ({
      states: {
        ...s.states,
        [tabId]: { ...ensureTab(s.states, tabId), filters: ensureTab(s.states, tabId).filters.filter((_, i) => i !== index), page: 1 },
      },
    })),

  setSorts: (tabId, sorts) =>
    set((s) => ({
      states: {
        ...s.states,
        [tabId]: { ...ensureTab(s.states, tabId), sorts },
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

  setChartConfig: (tabId, config) =>
    set((s) => ({
      states: {
        ...s.states,
        [tabId]: { ...ensureTab(s.states, tabId), chartConfig: config },
      },
    })),

  resetTab: (tabId) =>
    set((s) => {
      const { [tabId]: _, ...rest } = s.states;
      return { states: rest };
    }),
}));
