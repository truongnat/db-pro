import type { QueryTab } from "@/modules/query/state/query.store";

import type { WorkspaceTab } from "@/commons/types/workspace.types";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";

const MIGRATION_KEY = "db-pro-workspace-migrated";

function queryTabToWorkspace(tab: QueryTab, connectionId: string): WorkspaceTab {
  return {
    id: tab.id,
    kind: "query",
    title: tab.title,
    connectionId,
    resourceKey: `query:${tab.id}`,
    dirty: false,
    pinned: false,
    preview: false,
    order: Date.now() + Math.random(),
    data: {
      sql: tab.sql,
      status: tab.status,
      error: tab.error,
      result: null,
      explainPlan: null,
      sort: tab.sort,
      multiResults: null,
      multiResultIndex: 0,
    },
  };
}

export function migrateQueryTabsToWorkspace(
  tabs: QueryTab[],
  activeTabId: string,
  connectionId: string,
): boolean {
  if (sessionStorage.getItem(MIGRATION_KEY)) {
    return false;
  }

  const workspaceState = useWorkspaceStore.getState();
  if (workspaceState.tabs.length > 0) {
    sessionStorage.setItem(MIGRATION_KEY, "true");
    return false;
  }

  if (tabs.length === 0) {
    sessionStorage.setItem(MIGRATION_KEY, "true");
    return false;
  }

  const workspaceTabs = tabs.map((tab) => queryTabToWorkspace(tab, connectionId));

  useWorkspaceStore.getState().restoreState({
    tabs: workspaceTabs,
    activeTabId,
    recentlyClosed: [],
  });

  sessionStorage.setItem(MIGRATION_KEY, "true");
  return true;
}

export function hasMigrated(): boolean {
  return sessionStorage.getItem(MIGRATION_KEY) === "true";
}
