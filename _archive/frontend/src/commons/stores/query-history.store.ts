import { create } from "zustand";
import { persist } from "zustand/middleware";

export interface QueryHistoryStoreEntry {
  id: string;
  connectionId: string;
  sql: string;
  executedAt: string;
  durationMs: number;
  rowCount: number;
  status: "success" | "error";
  database: string | null;
  schema: string | null;
}

interface QueryHistoryState {
  entries: QueryHistoryStoreEntry[];
  favorites: Set<string>;
  maxEntries: number;

  addEntry: (entry: QueryHistoryStoreEntry) => void;
  clearHistory: () => void;
  clearByConnection: (connectionId: string) => void;
  toggleFavorite: (id: string) => void;
  isFavorite: (id: string) => boolean;
}

export const useQueryHistoryStore = create<QueryHistoryState>()(
  persist(
    (set, get) => ({
      entries: [],
      favorites: new Set<string>(),
      maxEntries: 1000,

      addEntry: (entry) =>
        set((state) => {
          const newEntries = [entry, ...state.entries].slice(0, state.maxEntries);
          return { entries: newEntries };
        }),

      clearHistory: () => set({ entries: [] }),

      clearByConnection: (connectionId) =>
        set((state) => ({
          entries: state.entries.filter((e) => e.connectionId !== connectionId),
        })),

      toggleFavorite: (id) =>
        set((state) => {
          const next = new Set(state.favorites);
          if (next.has(id)) {
            next.delete(id);
          } else {
            next.add(id);
          }
          return { favorites: next };
        }),

      isFavorite: (id) => {
        return get().favorites.has(id);
      },
    }),
    {
      name: "db-pro-query-history",
      partialize: (state) => ({
        entries: state.entries,
        favorites: Array.from(state.favorites),
      }),
      merge: (persisted, current) => {
        const p = persisted as (Partial<QueryHistoryState> & { favorites?: string[] }) | undefined;
        if (!p) return current;
        return {
          ...current,
          ...p,
          favorites: new Set(p.favorites ?? []),
        };
      },
    },
  ),
);
