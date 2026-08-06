import { create } from "zustand";

import type { ConnectionStatus } from "../types/connection.types";

interface ConnectionModuleState {
  statuses: Record<string, ConnectionStatus>;
  connectionErrors: Record<string, string>;

  setStatus: (id: string, status: ConnectionStatus) => void;
  setError: (id: string, error: string) => void;
  clearStatus: (id: string) => void;
  reset: () => void;
}

export const useConnectionModuleStore = create<ConnectionModuleState>()((set) => ({
  statuses: {},
  connectionErrors: {},

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

  reset: () => set({ statuses: {}, connectionErrors: {} }),
}));
