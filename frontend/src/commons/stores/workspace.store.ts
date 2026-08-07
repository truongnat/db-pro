import { create } from "zustand";
import { persist } from "zustand/middleware";

import type { PersistedWorkspaceState, QueryTabData, WorkspaceTab } from "@/commons/types/workspace.types";

const MAX_RECENTLY_CLOSED = 20;

interface WorkspaceState extends PersistedWorkspaceState {
  openTab: (tab: WorkspaceTab) => void;
  activateTab: (id: string) => void;
  closeTab: (id: string) => void;
  reopenLastClosed: () => void;
  closeOthers: (id: string) => void;
  closeRight: (id: string) => void;
  updateTabData: (id: string, updater: (data: QueryTabData) => QueryTabData) => void;
  setTabTitle: (id: string, title: string) => void;
  setTabDirty: (id: string, dirty: boolean) => void;
  toggleTabPinned: (id: string) => void;
  promotePreview: (id: string) => void;
  reorderTabs: (fromIndex: number, toIndex: number) => void;
  restoreState: (state: PersistedWorkspaceState) => void;
}

function findTabById(tabs: WorkspaceTab[], id: string): WorkspaceTab | undefined {
  return tabs.find((t) => t.id === id);
}

function updateTabInList(tabs: WorkspaceTab[], id: string, updater: (tab: WorkspaceTab) => WorkspaceTab): WorkspaceTab[] {
  return tabs.map((t) => (t.id === id ? updater(t) : t));
}

export const useWorkspaceStore = create<WorkspaceState>()(
  persist(
    (set, get) => ({
      tabs: [],
      activeTabId: null,
      recentlyClosed: [],

      openTab: (tab) =>
        set((state) => {
          const existing = state.tabs.find((t) => t.resourceKey === tab.resourceKey);
          if (existing) {
            if (existing.preview && !tab.preview) {
              return {
                tabs: updateTabInList(state.tabs, existing.id, (t) => ({ ...t, preview: false })),
                activeTabId: existing.id,
              };
            }
            return { activeTabId: existing.id };
          }

          if (tab.preview) {
            const previewIdx = state.tabs.findIndex((t) => t.preview && t.kind === tab.kind && t.connectionId === tab.connectionId);
            if (previewIdx !== -1) {
              const newTabs = [...state.tabs];
              newTabs[previewIdx] = { ...tab, id: newTabs[previewIdx].id, order: newTabs[previewIdx].order };
              return {
                tabs: newTabs,
                activeTabId: newTabs[previewIdx].id,
              };
            }
          }

          return {
            tabs: [...state.tabs, tab],
            activeTabId: tab.id,
          };
        }),

      activateTab: (id) =>
        set((state) => {
          if (!findTabById(state.tabs, id)) return state;
          return { activeTabId: id };
        }),

      closeTab: (id) =>
        set((state) => {
          const tab = findTabById(state.tabs, id);
          if (!tab || state.tabs.length <= 1 && !tab.pinned) return state;

          const idx = state.tabs.findIndex((t) => t.id === id);
          const newTabs = state.tabs.filter((t) => t.id !== id);
          let newActiveId = state.activeTabId;

          if (id === state.activeTabId) {
            const nextIdx = Math.min(idx, newTabs.length - 1);
            newActiveId = newTabs[nextIdx]?.id ?? null;
          }

          const recentlyClosed = tab.pinned
            ? state.recentlyClosed
            : [tab, ...state.recentlyClosed].slice(0, MAX_RECENTLY_CLOSED);

          return {
            tabs: newTabs,
            activeTabId: newActiveId,
            recentlyClosed,
          };
        }),

      reopenLastClosed: () =>
        set((state) => {
          if (state.recentlyClosed.length === 0) return state;
          const [tab, ...rest] = state.recentlyClosed;
          const reopened = { ...tab, id: crypto.randomUUID() };
          return {
            tabs: [...state.tabs, reopened],
            activeTabId: reopened.id,
            recentlyClosed: rest,
          };
        }),

      closeOthers: (id) =>
        set((state) => {
          const kept = state.tabs.filter((t) => t.id === id || t.pinned);
          if (kept.length === state.tabs.length) return state;
          return {
            tabs: kept,
            activeTabId: findTabById(kept, id) ? id : kept[0]?.id ?? null,
          };
        }),

      closeRight: (id) =>
        set((state) => {
          const idx = state.tabs.findIndex((t) => t.id === id);
          if (idx === -1) return state;
          const kept = state.tabs.filter((t, i) => i <= idx || t.pinned);
          return {
            tabs: kept,
            activeTabId: findTabById(kept, state.activeTabId ?? "") ? state.activeTabId : kept[kept.length - 1]?.id ?? null,
          };
        }),

      updateTabData: (id, updater) =>
        set((state) => ({
          tabs: updateTabInList(state.tabs, id, (t) =>
            t.kind === "query" ? { ...t, data: updater(t.data) } : t,
          ),
        })),

      setTabTitle: (id, title) =>
        set((state) => ({
          tabs: updateTabInList(state.tabs, id, (t) => ({ ...t, title })),
        })),

      setTabDirty: (id, dirty) =>
        set((state) => ({
          tabs: updateTabInList(state.tabs, id, (t) => ({ ...t, dirty })),
        })),

      toggleTabPinned: (id) =>
        set((state) => ({
          tabs: updateTabInList(state.tabs, id, (t) => ({ ...t, pinned: !t.pinned })),
        })),

      promotePreview: (id) =>
        set((state) => ({
          tabs: updateTabInList(state.tabs, id, (t) => ({ ...t, preview: false })),
        })),

      reorderTabs: (fromIndex, toIndex) =>
        set((state) => {
          if (fromIndex < 0 || fromIndex >= state.tabs.length) return state;
          if (toIndex < 0 || toIndex >= state.tabs.length) return state;
          const newTabs = [...state.tabs];
          const [moved] = newTabs.splice(fromIndex, 1);
          newTabs.splice(toIndex, 0, moved);
          return { tabs: newTabs };
        }),

      restoreState: (restored) =>
        set({
          tabs: restored.tabs,
          activeTabId: restored.activeTabId && findTabById(restored.tabs, restored.activeTabId)
            ? restored.activeTabId
            : restored.tabs[0]?.id ?? null,
          recentlyClosed: restored.recentlyClosed ?? [],
        }),
    }),
    {
      name: "db-pro-workspace",
      partialize: (state) => ({
        tabs: state.tabs.map((t) =>
          t.kind === "query"
            ? { ...t, data: { ...t.data, result: null, explainPlan: null, status: "idle" as const, error: null, multiResults: null } }
            : t,
        ),
        activeTabId: state.activeTabId,
        recentlyClosed: [],
      }),
    },
  ),
);
