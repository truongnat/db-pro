import type { PersistedWorkspaceState, QueryTabData, WorkspaceTab } from "@/commons/types/workspace.types";

/**
 * Current workspace schema version.
 * Increment when adding new migrations.
 */
export const CURRENT_WORKSPACE_VERSION = 2;

type MigrationFn = (state: PersistedWorkspaceState) => PersistedWorkspaceState;

/**
 * v0 → v1: Initial migration from unversioned state.
 * - Ensures workspaceVersion exists
 * - Migrates legacy "structure" section to "columns"
 * - Adds missing query context
 */
const migrateV0toV1: MigrationFn = (state) => {
  let changed = false;
  const migratedTabs: WorkspaceTab[] = state.tabs.map((tab) => {
    if (tab.kind === "db-object") {
      const activeSection = tab.data.activeSection as string;
      if (activeSection === "structure") {
        changed = true;
        return { ...tab, data: { ...tab.data, activeSection: "columns" } };
      }
      return tab;
    }
    const data = tab.data as QueryTabData;
    if (!data.context) {
      changed = true;
      return {
        ...tab,
        data: { ...data, context: { database: null, schema: null } },
      };
    }
    return tab;
  });
  return changed
    ? { ...state, workspaceVersion: 1, tabs: migratedTabs }
    : { ...state, workspaceVersion: 1 };
};

/**
 * v1 → v2: Add activePanel to query tabs if missing.
 */
const migrateV1toV2: MigrationFn = (state) => {
  let changed = false;
  const migratedTabs: WorkspaceTab[] = state.tabs.map((tab) => {
    if (tab.kind === "query") {
      const data = tab.data as QueryTabData;
      if (!data.activePanel) {
        changed = true;
        return { ...tab, data: { ...data, activePanel: "results" } };
      }
    }
    return tab;
  });
  return changed
    ? { ...state, workspaceVersion: 2, tabs: migratedTabs }
    : { ...state, workspaceVersion: 2 };
};

const migrations: Record<number, MigrationFn> = {
  0: migrateV0toV1,
  1: migrateV1toV2,
};

/**
 * Run all necessary migrations to bring the persisted workspace state
 * up to the current version.
 */
export function migrateWorkspace(state: PersistedWorkspaceState): PersistedWorkspaceState {
  const fromVersion = state.workspaceVersion ?? 0;
  if (fromVersion >= CURRENT_WORKSPACE_VERSION) return state;

  let current = { ...state, workspaceVersion: fromVersion };
  for (let v = fromVersion; v < CURRENT_WORKSPACE_VERSION; v++) {
    const migration = migrations[v];
    if (!migration) break;
    current = migration(current);
  }
  return current;
}

/**
 * Legacy migration function for backward compatibility.
 * Delegates to the versioned pipeline.
 * Returns the same array reference when no migration was needed.
 */
export function migratePersistedWorkspace(tabs: WorkspaceTab[]): WorkspaceTab[] {
  const state: PersistedWorkspaceState = {
    workspaceVersion: 0,
    tabs,
    activeTabId: null,
    recentlyClosed: [],
  };
  const migrated = migrateWorkspace(state);
  return migrated.tabs === tabs ? tabs : migrated.tabs;
}
