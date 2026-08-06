import { create } from "zustand";

interface QueryHistoryEntry {
  id: string;
  connectionId: string;
  sql: string;
  executedAt: string;
  durationMs: number;
  rowCount: number;
}

interface QueryHistoryState {
  entries: QueryHistoryEntry[];
  maxEntries: number;

  addEntry: (entry: QueryHistoryEntry) => void;
  clearHistory: () => void;
  clearByConnection: (connectionId: string) => void;
}

export const useQueryHistoryStore = create<QueryHistoryState>()((set) => ({
  entries: [],
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
}));
