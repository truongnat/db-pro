import { create } from "zustand";
import { persist } from "zustand/middleware";

/**
 * Snapshot of a dirty query tab's SQL content.
 * Used for crash recovery — if the app closes unexpectedly,
 * the next session can offer to restore unsaved SQL.
 */
export interface RecoverySnapshot {
  tabId: string;
  connectionId: string | null;
  title: string;
  sql: string;
  timestamp: number;
}

interface CrashRecoveryState {
  snapshots: RecoverySnapshot[];
  /** Save a dirty SQL snapshot for a tab. */
  saveSnapshot: (tabId: string, connectionId: string | null, title: string, sql: string) => void;
  /** Remove a snapshot (e.g. after user saved or restored). */
  removeSnapshot: (tabId: string) => void;
  /** Clear all recovery snapshots. */
  clearAll: () => void;
}

const MAX_SNAPSHOTS = 50;

export const useCrashRecoveryStore = create<CrashRecoveryState>()(
  persist(
    (set) => ({
      snapshots: [],

      saveSnapshot: (tabId, connectionId, title, sql) =>
        set((state) => {
          const filtered = state.snapshots.filter((s) => s.tabId !== tabId);
          const snapshot: RecoverySnapshot = {
            tabId,
            connectionId,
            title,
            sql,
            timestamp: Date.now(),
          };
          return {
            snapshots: [snapshot, ...filtered].slice(0, MAX_SNAPSHOTS),
          };
        }),

      removeSnapshot: (tabId) =>
        set((state) => ({
          snapshots: state.snapshots.filter((s) => s.tabId !== tabId),
        })),

      clearAll: () => set({ snapshots: [] }),
    }),
    {
      name: "db-pro-crash-recovery",
    },
  ),
);
