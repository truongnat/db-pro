import { create } from "zustand";
import { persist } from "zustand/middleware";

import type { Connection } from "@/modules/connection/types/connection.types";
import type { TranslatedError } from "@/commons/utils/error-types";

interface ConnectionState {
  connections: Connection[];
  explorerConnectionId: string | null;
  activeConnectionIds: string[];
  isLoading: boolean;
  error: TranslatedError | null;

  setConnections: (connections: Connection[]) => void;
  setExplorerConnection: (id: string | null) => void;
  setActiveConnection: (id: string) => void;
  removeActiveConnection: (id: string) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: TranslatedError | null) => void;
  addConnection: (connection: Connection) => void;
  updateConnection: (id: string, connection: Connection) => void;
  removeConnection: (id: string) => void;
  reset: () => void;
}

const initialState = {
  connections: [],
  explorerConnectionId: null,
  activeConnectionIds: [] as string[],
  isLoading: false,
  error: null,
};

export const useConnectionStore = create<ConnectionState>()(
  persist(
    (set) => ({
      ...initialState,

      setConnections: (connections) => set({ connections }),
      setExplorerConnection: (id) => set({ explorerConnectionId: id }),
      setActiveConnection: (id) =>
        set((state) => ({
          activeConnectionIds: state.activeConnectionIds.includes(id)
            ? state.activeConnectionIds
            : [...state.activeConnectionIds, id],
        })),
      removeActiveConnection: (id) =>
        set((state) => ({
          activeConnectionIds: state.activeConnectionIds.filter((cid) => cid !== id),
        })),
      setLoading: (loading) => set({ isLoading: loading }),
      setError: (error) => set({ error }),

      addConnection: (connection) =>
        set((state) => ({
          connections: [...state.connections, connection],
        })),

      updateConnection: (id, connection) =>
        set((state) => ({
          connections: state.connections.map((c) =>
            c.id === id ? connection : c,
          ),
        })),

      removeConnection: (id) =>
        set((state) => ({
          connections: state.connections.filter(
            (c) => c.id !== id,
          ),
          explorerConnectionId:
            state.explorerConnectionId === id ? null : state.explorerConnectionId,
          activeConnectionIds: state.activeConnectionIds.filter(
            (cid) => cid !== id,
          ),
        })),

      reset: () => set(initialState),
    }),
    {
      name: "db-pro-explorer-context",
      partialize: (state) => ({
        explorerConnectionId: state.explorerConnectionId,
        activeConnectionIds: state.activeConnectionIds,
      }),
    },
  ),
);
