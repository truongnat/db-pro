import { create } from "zustand";

import type { GridFilter, GridSort } from "../types/data-grid.types";

export interface ChartConfig {
  type: "bar" | "line" | "pie";
  xColumn: string;
  yColumn: string;
}

interface DataGridModuleState {
  connectionId: string | null;
  tableSchema: string | null;
  tableName: string | null;
  filters: GridFilter[];
  sorts: GridSort[];
  page: number;
  pageSize: number;
  editingCell: { row: number; col: number } | null;
  frozenColumns: string[];
  chartConfig: ChartConfig | null;

  setTable: (schema: string | null, name: string | null) => void;
  addFilter: (filter: GridFilter) => void;
  removeFilter: (index: number) => void;
  setSorts: (sorts: GridSort[]) => void;
  setPage: (page: number) => void;
  setPageSize: (size: number) => void;
  setEditingCell: (cell: { row: number; col: number } | null) => void;
  toggleFrozenColumn: (column: string) => void;
  setChartConfig: (config: ChartConfig | null) => void;
  reset: () => void;
}

const initialState = {
  connectionId: null as string | null,
  tableSchema: null as string | null,
  tableName: null as string | null,
  filters: [] as GridFilter[],
  sorts: [] as GridSort[],
  page: 1,
  pageSize: 50,
  editingCell: null as { row: number; col: number } | null,
  frozenColumns: [] as string[],
  chartConfig: null as ChartConfig | null,
};

export const useDataGridModuleStore = create<DataGridModuleState>()((set) => ({
  ...initialState,

  setTable: (schema, name) =>
    set({ tableSchema: schema, tableName: name, page: 1, filters: [], sorts: [], editingCell: null, frozenColumns: [], chartConfig: null }),

  addFilter: (filter) => set((s) => ({ filters: [...s.filters, filter], page: 1 })),

  removeFilter: (index) =>
    set((s) => ({ filters: s.filters.filter((_, i) => i !== index), page: 1 })),

  setSorts: (sorts) => set({ sorts }),

  setPage: (page) => set({ page }),

  setPageSize: (pageSize) => set({ pageSize, page: 1 }),

  setEditingCell: (cell) => set({ editingCell: cell }),

  toggleFrozenColumn: (column) =>
    set((s) => ({
      frozenColumns: s.frozenColumns.includes(column)
        ? s.frozenColumns.filter((c) => c !== column)
        : [...s.frozenColumns, column],
    })),

  setChartConfig: (config) => set({ chartConfig: config }),

  reset: () => set(initialState),
}));
