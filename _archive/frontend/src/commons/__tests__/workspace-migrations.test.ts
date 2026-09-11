import { describe, expect, it } from "vitest";

import {
  CURRENT_WORKSPACE_VERSION,
  migrateWorkspace,
  migratePersistedWorkspace,
} from "@/commons/services/workspace-migrations";
import type { PersistedWorkspaceState, WorkspaceTab } from "@/commons/types/workspace.types";

function makeQueryTab(overrides: Partial<WorkspaceTab> = {}): WorkspaceTab {
  return {
    id: "q1",
    kind: "query",
    title: "Query 1",
    connectionId: "c1",
    resourceKey: "query:c1:q1",
    dirty: false,
    pinned: false,
    preview: false,
    order: 0,
    data: {
      context: { database: null, schema: null },
      sql: "SELECT 1",
      status: "idle",
      error: null,
      result: null,
      explainPlan: null,
      sort: { column: null, direction: null },
      multiResults: null,
      multiResultIndex: 0,
      activePanel: "results",
    },
    ...overrides,
  } as WorkspaceTab;
}

function makeDbObjectTab(overrides: Record<string, unknown> = {}): WorkspaceTab {
  return {
    id: "db1",
    kind: "db-object",
    title: "users",
    connectionId: "c1",
    resourceKey: "db-object:c1:public:users",
    dirty: false,
    pinned: false,
    preview: false,
    order: 0,
    data: {
      schema: "public",
      objectName: "users",
      objectType: "table",
      activeSection: "columns",
    },
    ...overrides,
  } as WorkspaceTab;
}

describe("Workspace migrations", () => {
  describe("CURRENT_WORKSPACE_VERSION", () => {
    it("is 3", () => {
      expect(CURRENT_WORKSPACE_VERSION).toBe(3);
    });
  });

  describe("migrateWorkspace", () => {
    it("migrates v0 (unversioned) to current", () => {
      const state: PersistedWorkspaceState = {
        workspaceVersion: 0,
        tabs: [makeQueryTab()],
        activeTabId: "q1",
        recentlyClosed: [],
      };
      const result = migrateWorkspace(state);
      expect(result.workspaceVersion).toBe(CURRENT_WORKSPACE_VERSION);
    });

    it("does nothing when already at current version", () => {
      const state: PersistedWorkspaceState = {
        workspaceVersion: CURRENT_WORKSPACE_VERSION,
        tabs: [makeQueryTab()],
        activeTabId: "q1",
        recentlyClosed: [],
      };
      const result = migrateWorkspace(state);
      expect(result).toBe(state);
    });

    it("v0→v1: migrates 'structure' section to 'columns'", () => {
      const tab = makeDbObjectTab({
        data: {
          schema: "public",
          objectName: "users",
          objectType: "table",
          activeSection: "structure",
        },
      });
      const state: PersistedWorkspaceState = {
        workspaceVersion: 0,
        tabs: [tab],
        activeTabId: null,
        recentlyClosed: [],
      };
      const result = migrateWorkspace(state);
      expect(result.workspaceVersion).toBeGreaterThanOrEqual(1);
      const migratedTab = result.tabs[0];
      expect(migratedTab.kind).toBe("db-object");
      if (migratedTab.kind === "db-object") {
        expect(migratedTab.data.activeSection).toBe("columns");
      }
    });

    it("v0→v1: adds missing query context", () => {
      const tab = {
        ...makeQueryTab(),
        data: {
          ...makeQueryTab().data,
          context: undefined,
        },
      } as unknown as WorkspaceTab;
      const state: PersistedWorkspaceState = {
        workspaceVersion: 0,
        tabs: [tab],
        activeTabId: null,
        recentlyClosed: [],
      };
      const result = migrateWorkspace(state);
      const migratedTab = result.tabs[0];
      expect(migratedTab.kind).toBe("query");
      if (migratedTab.kind === "query") {
        expect(migratedTab.data.context).toEqual({ database: null, schema: null });
      }
    });

    it("v1→v2: adds missing activePanel to query tabs", () => {
      const tab = {
        ...makeQueryTab(),
        data: {
          ...makeQueryTab().data,
          activePanel: undefined,
        },
      } as unknown as WorkspaceTab;
      const state: PersistedWorkspaceState = {
        workspaceVersion: 1,
        tabs: [tab],
        activeTabId: null,
        recentlyClosed: [],
      };
      const result = migrateWorkspace(state);
      expect(result.workspaceVersion).toBe(CURRENT_WORKSPACE_VERSION);
      const migratedTab = result.tabs[0];
      if (migratedTab.kind === "query") {
        expect(migratedTab.data.activePanel).toBe("results");
      }
    });

    it("v2→v3: migrates 'diagram' section to 'columns'", () => {
      const tab = makeDbObjectTab({
        data: {
          schema: "public",
          objectName: "users",
          objectType: "table",
          activeSection: "diagram",
        },
      });
      const state: PersistedWorkspaceState = {
        workspaceVersion: 2,
        tabs: [tab],
        activeTabId: null,
        recentlyClosed: [],
      };
      const result = migrateWorkspace(state);
      expect(result.workspaceVersion).toBe(3);
      const migratedTab = result.tabs[0];
      if (migratedTab.kind === "db-object") {
        expect(migratedTab.data.activeSection).toBe("columns");
      }
    });

    it("v2→v3: does not affect tabs with other sections", () => {
      const tab = makeDbObjectTab({
        data: {
          schema: "public",
          objectName: "users",
          objectType: "table",
          activeSection: "data",
        },
      });
      const state: PersistedWorkspaceState = {
        workspaceVersion: 2,
        tabs: [tab],
        activeTabId: null,
        recentlyClosed: [],
      };
      const result = migrateWorkspace(state);
      expect(result.workspaceVersion).toBe(3);
      if (result.tabs[0].kind === "db-object") {
        expect(result.tabs[0].data.activeSection).toBe("data");
      }
    });

    it("preserves tab reference when no changes needed", () => {
      const tab = makeQueryTab();
      const state: PersistedWorkspaceState = {
        workspaceVersion: CURRENT_WORKSPACE_VERSION,
        tabs: [tab],
        activeTabId: null,
        recentlyClosed: [],
      };
      const result = migrateWorkspace(state);
      expect(result.tabs).toBe(state.tabs);
    });
  });

  describe("migratePersistedWorkspace (legacy)", () => {
    it("returns same reference when no migration needed", () => {
      const tabs = [makeQueryTab()];
      const result = migratePersistedWorkspace(tabs);
      expect(result).toBe(tabs);
    });

    it("migrates legacy section names", () => {
      const tabs = [
        makeDbObjectTab({
          data: {
            schema: "public",
            objectName: "users",
            objectType: "table",
            activeSection: "structure",
          },
        }),
      ];
      const result = migratePersistedWorkspace(tabs);
      expect(result).not.toBe(tabs);
      if (result[0].kind === "db-object") {
        expect(result[0].data.activeSection).toBe("columns");
      }
    });
  });
});
