import { create } from "zustand";
import { persist } from "zustand/middleware";

import {
  migrateWorkspace,
  CURRENT_WORKSPACE_VERSION,
} from "@/commons/services/workspace-migrations";
import { useConnectionStore } from "@/commons/stores/connection.store";
import type {
  DbObjectSection,
  PersistedWorkspaceState,
  QueryContext,
  QueryTabData,
  SchemaWorkspaceSection,
  WorkspaceTab,
} from "@/commons/types/workspace.types";
import { useTabGridStateStore } from "@/modules/data-grid/state/tab-grid-state.store";
import { useStagedChangesStore } from "@/modules/data-grid/state/staged-changes.store";

const MAX_RECENTLY_CLOSED = 20;

interface WorkspaceState extends PersistedWorkspaceState {
  openTab: (tab: WorkspaceTab) => void;
  activateTab: (id: string) => void;
  closeTab: (id: string) => void;
  reopenLastClosed: () => void;
  closeOthers: (id: string) => void;
  closeRight: (id: string) => void;
  closeAll: () => void;
  closeTabs: (ids: string[]) => void;
  updateTabData: (id: string, updater: (data: QueryTabData) => QueryTabData) => void;
  setTabTitle: (id: string, title: string) => void;
  setTabDirty: (id: string, dirty: boolean) => void;
  toggleTabPinned: (id: string) => void;
  promotePreview: (id: string) => void;
  reorderTabs: (fromIndex: number, toIndex: number) => void;
  restoreState: (state: PersistedWorkspaceState) => void;
  setDbObjectSection: (id: string, section: DbObjectSection) => void;
  setSchemaWorkspaceSection: (id: string, section: SchemaWorkspaceSection) => void;
  openDbObject: (tab: WorkspaceTab & { kind: "db-object" }) => void;
  setQueryTabConnection: (id: string, connectionId: string, context: QueryContext) => void;
  reassignTabConnection: (id: string, newConnectionId: string) => void;
}

function findTabById(tabs: WorkspaceTab[], id: string): WorkspaceTab | undefined {
  return tabs.find((t) => t.id === id);
}

function updateTabInList(
  tabs: WorkspaceTab[],
  id: string,
  updater: (tab: WorkspaceTab) => WorkspaceTab,
): WorkspaceTab[] {
  return tabs.map((t) => (t.id === id ? updater(t) : t));
}

function gcGridState() {
  const { tabs, recentlyClosed } = useWorkspaceStore.getState();
  const validIds = new Set<string>();
  for (const t of tabs) validIds.add(t.id);
  for (const t of recentlyClosed) validIds.add(t.id);
  useTabGridStateStore.getState().gc(validIds);
}

export const useWorkspaceStore = create<WorkspaceState>()(
  persist(
    (set, _get) => ({
      workspaceVersion: CURRENT_WORKSPACE_VERSION,
      tabs: [],
      activeTabId: null,
      recentlyClosed: [],

      openTab: (tab) => {
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
            const previewIdx = state.tabs.findIndex(
              (t) => t.preview && t.kind === tab.kind && t.connectionId === tab.connectionId,
            );
            if (previewIdx !== -1) {
              const existingPreview = state.tabs[previewIdx];
              const hasStaged = useStagedChangesStore.getState().getCount(existingPreview.id) > 0;
              if (hasStaged) {
                const promoted = [...state.tabs];
                promoted[previewIdx] = { ...existingPreview, preview: false };
                return {
                  tabs: [...promoted, tab],
                  activeTabId: tab.id,
                };
              }
              const newTabs = [...state.tabs];
              newTabs[previewIdx] = {
                ...tab,
                id: newTabs[previewIdx].id,
                order: newTabs[previewIdx].order,
              };
              useTabGridStateStore.getState().resetTab(newTabs[previewIdx].id);
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
        });
      },

      activateTab: (id) =>
        set((state) => {
          if (!findTabById(state.tabs, id)) return state;
          return { activeTabId: id };
        }),

      closeTab: (id) => {
        set((state) => {
          const tab = findTabById(state.tabs, id);
          if (!tab) return state;

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
        });
        gcGridState();
      },

      reopenLastClosed: () =>
        set((state) => {
          if (state.recentlyClosed.length === 0) return state;
          const [tab, ...rest] = state.recentlyClosed;
          const reopened = { ...tab };
          return {
            tabs: [...state.tabs, reopened],
            activeTabId: reopened.id,
            recentlyClosed: rest,
          };
        }),

      closeOthers: (id) => {
        set((state) => {
          const evicted = state.tabs.filter((t) => t.id !== id && !t.pinned);
          const kept = state.tabs.filter((t) => t.id === id || t.pinned);
          if (evicted.length === 0) return state;
          const recentlyClosed = [...evicted.reverse(), ...state.recentlyClosed].slice(
            0,
            MAX_RECENTLY_CLOSED,
          );
          return {
            tabs: kept,
            activeTabId: findTabById(kept, id) ? id : (kept[0]?.id ?? null),
            recentlyClosed,
          };
        });
        gcGridState();
      },

      closeRight: (id) => {
        set((state) => {
          const idx = state.tabs.findIndex((t) => t.id === id);
          if (idx === -1) return state;
          const evicted = state.tabs.filter((t, i) => i > idx && !t.pinned);
          const kept = state.tabs.filter((t, i) => i <= idx || t.pinned);
          if (evicted.length === 0) return state;
          const recentlyClosed = [...evicted.reverse(), ...state.recentlyClosed].slice(
            0,
            MAX_RECENTLY_CLOSED,
          );
          return {
            tabs: kept,
            activeTabId: findTabById(kept, state.activeTabId ?? "")
              ? state.activeTabId
              : (kept[kept.length - 1]?.id ?? null),
            recentlyClosed,
          };
        });
        gcGridState();
      },

      closeAll: () => {
        set((state) => {
          const evicted = state.tabs.filter((t) => !t.pinned);
          const kept = state.tabs.filter((t) => t.pinned);
          if (evicted.length === 0) return state;
          const recentlyClosed = [...evicted.reverse(), ...state.recentlyClosed].slice(
            0,
            MAX_RECENTLY_CLOSED,
          );
          return {
            tabs: kept,
            activeTabId: kept[0]?.id ?? null,
            recentlyClosed,
          };
        });
        gcGridState();
      },

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
          const pinnedCount = state.tabs.filter((t) => t.pinned).length;
          if (fromIndex < pinnedCount || toIndex < pinnedCount) return state;
          if (fromIndex >= state.tabs.length || toIndex >= state.tabs.length) return state;
          const newTabs = [...state.tabs];
          const [moved] = newTabs.splice(fromIndex, 1);
          newTabs.splice(toIndex, 0, moved);
          return { tabs: newTabs };
        }),

      closeTabs: (ids) => {
        set((state) => {
          const idSet = new Set(ids);
          const evicted = state.tabs.filter((t) => idSet.has(t.id) && !t.pinned);
          if (evicted.length === 0) return state;
          const kept = state.tabs.filter((t) => !idSet.has(t.id) || t.pinned);
          const recentlyClosed = [...evicted.reverse(), ...state.recentlyClosed].slice(
            0,
            MAX_RECENTLY_CLOSED,
          );
          let newActiveId = state.activeTabId;
          if (state.activeTabId && idSet.has(state.activeTabId)) {
            const closedIdx = state.tabs.findIndex((t) => t.id === state.activeTabId);
            const nextIdx = Math.min(closedIdx, kept.length - 1);
            newActiveId = kept[nextIdx]?.id ?? null;
          }
          return {
            tabs: kept,
            activeTabId: newActiveId,
            recentlyClosed,
          };
        });
        gcGridState();
      },

      setDbObjectSection: (id, section) =>
        set((state) => ({
          tabs: updateTabInList(state.tabs, id, (t) =>
            t.kind === "db-object" ? { ...t, data: { ...t.data, activeSection: section } } : t,
          ),
        })),

      setSchemaWorkspaceSection: (id, section) =>
        set((state) => ({
          tabs: updateTabInList(state.tabs, id, (t) =>
            t.kind === "schema-workspace"
              ? { ...t, data: { ...t.data, activeSection: section } }
              : t,
          ),
        })),

      setQueryTabConnection: (id, connectionId, context) =>
        set((state) => ({
          tabs: updateTabInList(state.tabs, id, (t) =>
            t.kind === "query"
              ? {
                  ...t,
                  connectionId,
                  data: {
                    ...t.data,
                    context,
                    status: "idle",
                    error: null,
                    result: null,
                    explainPlan: null,
                    multiResults: null,
                    multiResultIndex: 0,
                    activeExecutionId: null,
                    executionStartedAt: null,
                  },
                }
              : t,
          ),
        })),

      openDbObject: (tab) => {
        set((state) => {
          const existing = state.tabs.find((t) => t.resourceKey === tab.resourceKey);
          if (existing && existing.kind === "db-object") {
            const shouldPromote = existing.preview && !tab.preview;
            return {
              tabs: updateTabInList(state.tabs, existing.id, (t) =>
                shouldPromote && t.kind === "db-object" ? { ...t, preview: false } : t,
              ),
              activeTabId: existing.id,
            };
          }

          if (tab.preview) {
            const previewIdx = state.tabs.findIndex(
              (t) => t.preview && t.kind === "db-object" && t.connectionId === tab.connectionId,
            );
            if (previewIdx !== -1) {
              const existingPreview = state.tabs[previewIdx];
              const hasStaged = useStagedChangesStore.getState().getCount(existingPreview.id) > 0;
              if (hasStaged) {
                const promoted = [...state.tabs];
                promoted[previewIdx] = { ...existingPreview, preview: false };
                return {
                  tabs: [...promoted, tab],
                  activeTabId: tab.id,
                };
              }
              const reusedId = state.tabs[previewIdx].id;
              const newTabs = [...state.tabs];
              newTabs[previewIdx] = { ...tab, id: reusedId, order: newTabs[previewIdx].order };
              useTabGridStateStore.getState().resetTab(reusedId);
              return {
                tabs: newTabs,
                activeTabId: reusedId,
              };
            }
          }

          return {
            tabs: [...state.tabs, tab],
            activeTabId: tab.id,
          };
        });
      },

      reassignTabConnection: (id, newConnectionId) =>
        set((state) => {
          // Clear staged changes — they reference the old connection's backend.
          useStagedChangesStore.getState().clearTab(id);

          const connections = useConnectionStore.getState().connections;
          const newConn = connections.find((c) => c.id === newConnectionId);
          const targetIsSqlite = newConn?.driver === "sqlite";

          return {
            tabs: updateTabInList(state.tabs, id, (t) => {
              const sourceConn = connections.find((c) => c.id === t.connectionId);
              const sourceIsSqlite = sourceConn?.driver === "sqlite";
              const normalizeSchema = (schema: string | null | undefined): string => {
                if (targetIsSqlite) return "main";
                if (!schema) return "public";
                if (sourceIsSqlite && schema === "main") return "public";
                return schema;
              };

              if (t.kind === "schema-workspace") {
                const schema = normalizeSchema(t.data.schema);
                return {
                  ...t,
                  title: `ER: ${schema}`,
                  connectionId: newConnectionId,
                  resourceKey: `schema-ws:${schema}:${newConnectionId}`,
                  dirty: true,
                  data: {
                    ...t.data,
                    schema,
                  },
                };
              }
              if (t.kind === "db-object") {
                const schema = normalizeSchema(t.data.schema);
                return {
                  ...t,
                  connectionId: newConnectionId,
                  resourceKey: `dbobj:${schema}.${t.data.objectName}:${newConnectionId}`,
                  dirty: true,
                  data: {
                    ...t.data,
                    schema,
                  },
                };
              }
              // Query tab: reset context to new connection's default database and schema
              const defaultDb = newConn?.database ?? null;
              const defaultSchema = targetIsSqlite ? "main" : "public";
              return {
                ...t,
                connectionId: newConnectionId,
                dirty: true,
                data: {
                  ...t.data,
                  context: { database: defaultDb, schema: defaultSchema },
                  status: "idle" as const,
                  error: null,
                  result: null,
                  explainPlan: null,
                  multiResults: null,
                  multiResultIndex: 0,
                  activeExecutionId: null,
                  executionStartedAt: null,
                  timing: null,
                },
              };
            }),
          };
        }),

      restoreState: (restored) => {
        set({
          tabs: restored.tabs,
          activeTabId:
            restored.activeTabId && findTabById(restored.tabs, restored.activeTabId)
              ? restored.activeTabId
              : (restored.tabs[0]?.id ?? null),
          recentlyClosed: restored.recentlyClosed ?? [],
        });
        gcGridState();
      },
    }),
    {
      name: "db-pro-workspace",
      partialize: (state) => ({
        workspaceVersion: CURRENT_WORKSPACE_VERSION,
        tabs: state.tabs.map((t) =>
          t.kind === "query"
            ? {
                ...t,
                data: {
                  ...t.data,
                  result: null,
                  explainPlan: null,
                  status: "idle" as const,
                  error: null,
                  multiResults: null,
                  activeExecutionId: null,
                  executionStartedAt: null,
                },
              }
            : t,
        ),
        activeTabId: state.activeTabId,
        recentlyClosed: [],
      }),
      onRehydrateStorage: () => (state) => {
        if (!state) return;

        // Run versioned migrations
        const persisted: PersistedWorkspaceState = {
          workspaceVersion: state.workspaceVersion ?? 0,
          tabs: state.tabs,
          activeTabId: state.activeTabId,
          recentlyClosed: state.recentlyClosed ?? [],
        };
        const migrated = migrateWorkspace(persisted);

        // IMPORTANT: Do NOT prune tabs here based on connections.
        // At rehydrate time, the connection list is still empty (loaded async).
        // Pruning here would delete all persisted tabs — a data-loss race.
        // Instead, call reconcileWorkspaceTabs() after connections are loaded.
        if (migrated.workspaceVersion !== persisted.workspaceVersion) {
          useWorkspaceStore.setState({
            workspaceVersion: CURRENT_WORKSPACE_VERSION,
            tabs: migrated.tabs,
            activeTabId: migrated.activeTabId,
            recentlyClosed: migrated.recentlyClosed,
          });
        }
      },
    },
  ),
);

/**
 * Reconcile persisted workspace tabs against the now-loaded connection list.
 *
 * Call this AFTER connections have been fetched (e.g. from useConnectionList onSuccess).
 * Tabs referencing connections that no longer exist are preserved but marked orphan
 * via the WorkspaceContent OrphanedTabView — they are NOT deleted, so the user can
 * decide what to do. Only tabs with empty connectionId are always kept.
 */
export function reconcileWorkspaceTabs(): void {
  const { tabs, activeTabId } = useWorkspaceStore.getState();
  const connections = useConnectionStore.getState().connections;
  void connections; // reserved for future reconciliation logic

  // We don't delete orphaned tabs — WorkspaceContent already shows OrphanedTabView
  // for tabs whose connectionId doesn't match any known connection. This is the
  // safe path: the user sees the tab and can close it manually.
  // The only thing we fix here is the activeTabId if it pointed to a now-removed tab.
  const activeStillValid = activeTabId && tabs.some((t) => t.id === activeTabId);
  if (!activeStillValid && tabs.length > 0) {
    useWorkspaceStore.setState({ activeTabId: tabs[0].id });
  }
}
