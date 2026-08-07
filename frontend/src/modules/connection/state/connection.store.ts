import { create } from "zustand";

import type { ConnectionStatus } from "../types/connection.types";

export type ConnectionSortField = "name" | "driver" | "group" | "recent";
export type ConnectionSortDirection = "asc" | "desc";

interface ConnectionModuleState {
  statuses: Record<string, ConnectionStatus>;
  connectionErrors: Record<string, string>;
  favorites: Record<string, boolean>;
  sortField: ConnectionSortField;
  sortDirection: ConnectionSortDirection;
  filterTag: string | null;
  filterGroup: string | null;

  setStatus: (id: string, status: ConnectionStatus) => void;
  setError: (id: string, error: string) => void;
  clearStatus: (id: string) => void;
  toggleFavorite: (id: string) => void;
  setSortField: (field: ConnectionSortField) => void;
  setSortDirection: (dir: ConnectionSortDirection) => void;
  setFilterTag: (tag: string | null) => void;
  setFilterGroup: (group: string | null) => void;
  clearFilters: () => void;
  reset: () => void;
}

const initialState = {
  statuses: {} as Record<string, ConnectionStatus>,
  connectionErrors: {} as Record<string, string>,
  favorites: {} as Record<string, boolean>,
  sortField: "name" as ConnectionSortField,
  sortDirection: "asc" as ConnectionSortDirection,
  filterTag: null as string | null,
  filterGroup: null as string | null,
};

export const useConnectionModuleStore = create<ConnectionModuleState>()((set) => ({
  ...initialState,

  setStatus: (id, status) =>
    set((state) => ({
      statuses: { ...state.statuses, [id]: status },
    })),

  setError: (id, error) =>
    set((state) => ({
      connectionErrors: { ...state.connectionErrors, [id]: error },
    })),

  clearStatus: (id) =>
    set((state) => {
      const { [id]: _s, ...restStatuses } = state.statuses;
      const { [id]: _e, ...restErrors } = state.connectionErrors;
      return { statuses: restStatuses, connectionErrors: restErrors };
    }),

  toggleFavorite: (id) =>
    set((state) => ({
      favorites: { ...state.favorites, [id]: !state.favorites[id] },
    })),

  setSortField: (field) => set({ sortField: field }),
  setSortDirection: (dir) => set({ sortDirection: dir }),
  setFilterTag: (tag) => set({ filterTag: tag }),
  setFilterGroup: (group) => set({ filterGroup: group }),
  clearFilters: () => set({ filterTag: null, filterGroup: null }),
  reset: () => set(initialState),
}));
