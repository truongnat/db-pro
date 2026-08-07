import { create } from "zustand";
import { persist } from "zustand/middleware";

import type { RecentResource } from "@/commons/types/quick-open.types";

const MAX_RECENT = 10;
const MAX_RECENT_RESOURCES = 20;

export interface RecentConnection {
  connectionId: string;
  lastConnectedAt: string;
  connectCount: number;
}

interface RecentState {
  recentConnections: RecentConnection[];
  recentResources: RecentResource[];
  connectionDialogOpen: boolean;
  connectionDialogEditId: string | null;

  addRecentConnection: (connectionId: string) => void;
  removeRecentConnection: (connectionId: string) => void;
  addRecentResource: (resource: Omit<RecentResource, "openedAt">) => void;
  removeRecentResource: (resourceKey: string) => void;
  openConnectionDialog: (editId?: string) => void;
  closeConnectionDialog: () => void;
}

export const useRecentStore = create<RecentState>()(
  persist(
    (set) => ({
      recentConnections: [],
      recentResources: [],
      connectionDialogOpen: false,
      connectionDialogEditId: null,

      addRecentConnection: (connectionId) =>
        set((state) => {
          const existing = state.recentConnections.find(
            (rc) => rc.connectionId === connectionId,
          );
          const now = new Date().toISOString();

          let updated: RecentConnection[];
          if (existing) {
            updated = [
              { ...existing, lastConnectedAt: now, connectCount: existing.connectCount + 1 },
              ...state.recentConnections.filter((rc) => rc.connectionId !== connectionId),
            ];
          } else {
            updated = [
              { connectionId, lastConnectedAt: now, connectCount: 1 },
              ...state.recentConnections,
            ];
          }

          return { recentConnections: updated.slice(0, MAX_RECENT) };
        }),

      removeRecentConnection: (connectionId) =>
        set((state) => ({
          recentConnections: state.recentConnections.filter(
            (rc) => rc.connectionId !== connectionId,
          ),
        })),

      addRecentResource: (resource) =>
        set((state) => {
          const now = new Date().toISOString();
          const updated = [
            { ...resource, openedAt: now },
            ...state.recentResources.filter(
              (r) => r.resourceKey !== resource.resourceKey,
            ),
          ];
          return { recentResources: updated.slice(0, MAX_RECENT_RESOURCES) };
        }),

      removeRecentResource: (resourceKey) =>
        set((state) => ({
          recentResources: state.recentResources.filter(
            (r) => r.resourceKey !== resourceKey,
          ),
        })),

      openConnectionDialog: (editId) =>
        set({
          connectionDialogOpen: true,
          connectionDialogEditId: editId ?? null,
        }),

      closeConnectionDialog: () =>
        set({
          connectionDialogOpen: false,
          connectionDialogEditId: null,
        }),
    }),
    {
      name: "db-pro-recent",
      partialize: (state) => ({
        recentConnections: state.recentConnections,
        recentResources: state.recentResources,
      }),
    },
  ),
);
