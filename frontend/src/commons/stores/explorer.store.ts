import { create } from "zustand";
import { persist } from "zustand/middleware";

interface ExplorerState {
  expandedNodes: string[];
  filter: string;
  toggleNode: (path: string) => void;
  expandNode: (path: string) => void;
  setFilter: (filter: string) => void;
  collapseAll: () => void;
}

export const useExplorerStore = create<ExplorerState>()(
  persist(
    (set) => ({
      expandedNodes: [],
      filter: "",

      toggleNode: (path) =>
        set((state) => {
          const has = state.expandedNodes.includes(path);
          return {
            expandedNodes: has
              ? state.expandedNodes.filter((p) => p !== path)
              : [...state.expandedNodes, path],
          };
        }),

      expandNode: (path) =>
        set((state) => {
          if (state.expandedNodes.includes(path)) return state;
          return { expandedNodes: [...state.expandedNodes, path] };
        }),

      setFilter: (filter) => set({ filter }),
      collapseAll: () => set({ expandedNodes: [] }),
    }),
    {
      name: "db-pro-explorer",
    },
  ),
);
