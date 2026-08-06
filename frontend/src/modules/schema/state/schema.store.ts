import { create } from "zustand";

import type { DetailTab } from "../types/schema.types";

interface SchemaModuleState {
  selectedSchema: string | null;
  selectedTable: string | null;
  selectedNodeType: "table" | "view" | null;
  expandedNodes: Set<string>;
  activeTab: DetailTab;
  searchQuery: string;

  setSelectedTable: (
    schema: string | null,
    table: string | null,
    nodeType: "table" | "view" | null,
  ) => void;
  toggleNode: (nodeId: string) => void;
  setActiveTab: (tab: DetailTab) => void;
  setSearchQuery: (query: string) => void;
  reset: () => void;
}

const initialState = {
  selectedSchema: null,
  selectedTable: null,
  selectedNodeType: null as "table" | "view" | null,
  expandedNodes: new Set<string>(),
  activeTab: "columns" as DetailTab,
  searchQuery: "",
};

export const useSchemaModuleStore = create<SchemaModuleState>()((set) => ({
  ...initialState,

  setSelectedTable: (schema, table, nodeType) =>
    set({
      selectedSchema: schema,
      selectedTable: table,
      selectedNodeType: nodeType,
      activeTab: "columns",
    }),

  toggleNode: (nodeId) =>
    set((state) => {
      const next = new Set(state.expandedNodes);
      if (next.has(nodeId)) {
        next.delete(nodeId);
      } else {
        next.add(nodeId);
      }
      return { expandedNodes: next };
    }),

  setActiveTab: (tab) => set({ activeTab: tab }),

  setSearchQuery: (query) => set({ searchQuery: query }),

  reset: () => set(initialState),
}));
