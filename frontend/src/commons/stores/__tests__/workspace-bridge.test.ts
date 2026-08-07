import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { migrateQueryTabsToWorkspace, hasMigrated } from "@/commons/stores/workspace-bridge";
import type { QueryTab } from "@/modules/query/state/query.store";

function resetWorkspace() {
  useWorkspaceStore.setState({
    tabs: [],
    activeTabId: null,
    recentlyClosed: [],
  });
}

function makeQueryTab(overrides: Partial<QueryTab> = {}): QueryTab {
  return {
    id: "tab-1",
    title: "Query 1",
    sql: "SELECT 1",
    status: "idle",
    error: null,
    result: null,
    explainPlan: null,
    sort: { column: null, direction: null },
    multiResults: null,
    multiResultIndex: 0,
    ...overrides,
  };
}

describe("workspace-bridge", () => {
  beforeEach(() => {
    resetWorkspace();
    sessionStorage.clear();
  });

  afterEach(() => {
    resetWorkspace();
    sessionStorage.clear();
  });

  describe("migrateQueryTabsToWorkspace", () => {
    it("migrates legacy tabs to workspace store", () => {
      const tabs = [
        makeQueryTab({ id: "tab-1", title: "Query 1", sql: "SELECT 1" }),
        makeQueryTab({ id: "tab-2", title: "Query 2", sql: "SELECT 2" }),
      ];

      const result = migrateQueryTabsToWorkspace(tabs, "tab-1", "conn-1");

      expect(result).toBe(true);
      const state = useWorkspaceStore.getState();
      expect(state.tabs).toHaveLength(2);
      expect(state.activeTabId).toBe("tab-1");
      expect(state.tabs[0].kind).toBe("query");
      expect(state.tabs[0].connectionId).toBe("conn-1");
      expect(state.tabs[0].data.sql).toBe("SELECT 1");
    });

    it("strips transient data (result, explainPlan) during migration", () => {
      const tabs = [
        makeQueryTab({
          id: "tab-1",
          result: { columns: [], rows: [], rowCount: 0, durationMs: 10 },
          explainPlan: { plan: "test" },
          status: "success",
          error: "some error",
        }),
      ];

      migrateQueryTabsToWorkspace(tabs, "tab-1", "conn-1");

      const state = useWorkspaceStore.getState();
      expect(state.tabs[0].data.result).toBeNull();
      expect(state.tabs[0].data.explainPlan).toBeNull();
      expect(state.tabs[0].data.status).toBe("success");
      expect(state.tabs[0].data.error).toBe("some error");
    });

    it("skips migration if workspace already has tabs", () => {
      const existingTab = {
        id: "existing",
        kind: "query" as const,
        title: "Existing",
        connectionId: "conn-1",
        resourceKey: "query:existing",
        dirty: false,
        pinned: false,
        preview: false,
        order: 1,
        data: {
          sql: "SELECT 'existing'",
          status: "idle" as const,
          error: null,
          result: null,
          explainPlan: null,
          sort: { column: null, direction: null as const },
          multiResults: null,
          multiResultIndex: 0,
        },
      };
      useWorkspaceStore.setState({ tabs: [existingTab] });

      const tabs = [makeQueryTab()];
      const result = migrateQueryTabsToWorkspace(tabs, "tab-1", "conn-1");

      expect(result).toBe(false);
      expect(useWorkspaceStore.getState().tabs).toHaveLength(1);
      expect(useWorkspaceStore.getState().tabs[0].id).toBe("existing");
    });

    it("skips migration if no legacy tabs", () => {
      const result = migrateQueryTabsToWorkspace([], "tab-1", "conn-1");
      expect(result).toBe(false);
      expect(useWorkspaceStore.getState().tabs).toHaveLength(0);
    });

    it("sets migrated flag in sessionStorage", () => {
      migrateQueryTabsToWorkspace([makeQueryTab()], "tab-1", "conn-1");
      expect(hasMigrated()).toBe(true);
    });

    it("does not re-migrate on subsequent calls", () => {
      const tabs1 = [makeQueryTab({ id: "tab-1", sql: "SELECT 'first'" })];
      migrateQueryTabsToWorkspace(tabs1, "tab-1", "conn-1");

      resetWorkspace();

      const tabs2 = [makeQueryTab({ id: "tab-2", sql: "SELECT 'second'" })];
      const result = migrateQueryTabsToWorkspace(tabs2, "tab-2", "conn-1");

      expect(result).toBe(false);
      expect(useWorkspaceStore.getState().tabs).toHaveLength(0);
    });

    it("preserves resourceKey format as query:{id}", () => {
      const tabs = [makeQueryTab({ id: "my-tab" })];
      migrateQueryTabsToWorkspace(tabs, "my-tab", "conn-1");

      const state = useWorkspaceStore.getState();
      expect(state.tabs[0].resourceKey).toBe("query:my-tab");
    });

    it("sets all migrated tabs as non-dirty, non-pinned, non-preview", () => {
      const tabs = [makeQueryTab()];
      migrateQueryTabsToWorkspace(tabs, "tab-1", "conn-1");

      const tab = useWorkspaceStore.getState().tabs[0];
      expect(tab.dirty).toBe(false);
      expect(tab.pinned).toBe(false);
      expect(tab.preview).toBe(false);
    });
  });

  describe("hasMigrated", () => {
    it("returns false before any migration", () => {
      expect(hasMigrated()).toBe(false);
    });

    it("returns true after migration", () => {
      migrateQueryTabsToWorkspace([makeQueryTab()], "tab-1", "conn-1");
      expect(hasMigrated()).toBe(true);
    });
  });
});
