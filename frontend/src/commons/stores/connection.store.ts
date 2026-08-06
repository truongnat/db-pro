import { create } from "zustand";
import { persist } from "zustand/middleware";

import type { TranslatedError } from "@/commons/utils/error-types";

interface ConnectionState {
  connections: unknown[];
  activeConnectionId: string | null;
  isLoading: boolean;
  error: TranslatedError | null;

  setConnections: (connections: unknown[]) => void;
  setActiveConnection: (id: string | null) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: TranslatedError | null) => void;
  addConnection: (connection: unknown) => void;
  updateConnection: (id: string, connection: unknown) => void;
  removeConnection: (id: string) => void;
  reset: () => void;
}

const initialState = {
  connections: [],
  activeConnectionId: null,
  isLoading: false,
  error: null,
};

export const useConnectionStore = create<ConnectionState>()(
  persist(
    (set) => ({
      ...initialState,

      setConnections: (connections) => set({ connections }),
      setActiveConnection: (id) => set({ activeConnectionId: id }),
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
          activeConnectionId:
            state.activeConnectionId === id ? null : state.activeConnectionId,
        })),

      reset: () => set(initialState),
    }),
    {
      name: "db-pro-connection",
      partialize: (state) => ({
        activeConnectionId: state.activeConnectionId,
      }),
    },
  ),
);
