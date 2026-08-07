import { create } from "zustand";
import { persist } from "zustand/middleware";

import type { TranslatedError } from "@/commons/utils/error-types";

interface ConnectionState {
  connections: unknown[];
  explorerConnectionId: string | null;
  isLoading: boolean;
  error: TranslatedError | null;

  setConnections: (connections: unknown[]) => void;
  setExplorerConnection: (id: string | null) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: TranslatedError | null) => void;
  addConnection: (connection: unknown) => void;
  updateConnection: (id: string, connection: unknown) => void;
  removeConnection: (id: string) => void;
  reset: () => void;
}

const initialState = {
  connections: [],
  explorerConnectionId: null,
  isLoading: false,
  error: null,
};

export const useConnectionStore = create<ConnectionState>()(
  persist(
    (set) => ({
      ...initialState,

      setConnections: (connections) => set({ connections }),
      setExplorerConnection: (id) => set({ explorerConnectionId: id }),
      setLoading: (loading) => set({ isLoading: loading }),
      setError: (error) => set({ error }),

      addConnection: (connection) =>
        set((state) => ({
          connections: [...state.connections, connection],
        })),

      updateConnection: (id, connection) =>
        set((state) => ({
          connections: state.connections.map((c) =>
            (c as { id: string }).id === id ? connection : c,
          ),
        })),

      removeConnection: (id) =>
        set((state) => ({
          connections: state.connections.filter(
            (c) => (c as { id: string }).id !== id,
          ),
          explorerConnectionId:
            state.explorerConnectionId === id ? null : state.explorerConnectionId,
        })),

      reset: () => set(initialState),
    }),
    {
      name: "db-pro-explorer-context",
      partialize: (state) => ({
        explorerConnectionId: state.explorerConnectionId,
      }),
    },
  ),
);
