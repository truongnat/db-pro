import { describe, it, expect, beforeEach } from "vitest";

import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { useConnectionStore } from "@/commons/stores/connection.store";
import { useCloseGuardStore } from "@/commons/stores/close-guard.store";
import { useStagedChangesStore } from "@/modules/data-grid/state/staged-changes.store";
import { requestCloseTab } from "@/commons/services/request-close-tab";
import type { Connection } from "@/modules/connection/types/connection.types";

const mockPgConnection: Connection = {
  id: "conn-pg",
  name: "PostgreSQL DB",
  driver: "postgres",
  host: "localhost",
  port: 5432,
  database: "postgres",
  username: "postgres",
  readOnly: false,
};

const mockSqliteConnection: Connection = {
  id: "conn-sqlite",
  name: "SQLite DB",
  driver: "sqlite",
  database: "/tmp/test.db",
  readOnly: false,
};

describe("QA-P1-10 & QA-P1-11 Workspace Recovery Remediation", () => {
  beforeEach(() => {
    useWorkspaceStore.setState({
      tabs: [],
      activeTabId: null,
      recentlyClosed: [],
    });
    useConnectionStore.setState({
      connections: [mockPgConnection, mockSqliteConnection],
    });
    useCloseGuardStore.setState({
      open: false,
      tabIds: [],
      dirtyCount: 0,
    });
    useStagedChangesStore.setState({
      changes: {},
      inFlightIds: new Set<string>(),
    });
  });

  describe("QA-P1-10: Orphan dirty tab close guard", () => {
    it("opens close guard dialog when closing orphaned tab with dirty SQL", () => {
      useWorkspaceStore.setState({
        tabs: [
          {
            id: "tab-orphan-1",
            kind: "query",
            title: "Orphaned Query",
            connectionId: "conn-deleted",
            dirty: true,
            resourceKey: "query:tab-orphan-1",
            data: {
              sql: "SELECT * FROM unsaved_query",
              status: "idle",
              error: null,
              result: null,
              explainPlan: null,
              multiResults: null,
              multiResultIndex: 0,
              activeExecutionId: null,
              executionStartedAt: null,
            },
          },
        ],
        activeTabId: "tab-orphan-1",
      });

      requestCloseTab("tab-orphan-1");

      const closeGuardState = useCloseGuardStore.getState();
      expect(closeGuardState.open).toBe(true);
      expect(closeGuardState.tabIds).toEqual(["tab-orphan-1"]);
      expect(closeGuardState.dirtyCount).toBe(1);
    });

    it("opens close guard dialog when closing orphaned tab with staged grid edits", () => {
      useWorkspaceStore.setState({
        tabs: [
          {
            id: "tab-orphan-2",
            kind: "db-object",
            title: "Orphaned Table Data",
            connectionId: "conn-deleted",
            dirty: false,
            resourceKey: "dbobj:public.users:conn-deleted",
            data: {
              schema: "public",
              objectName: "users",
              objectType: "table",
              activeSection: "data",
            },
          },
        ],
        activeTabId: "tab-orphan-2",
      });

      useStagedChangesStore.getState().stageCellEdit("tab-orphan-2", {
        pkValues: [{ type: "int64", value: "1" }],
        changes: { name: { type: "text", value: "Bob" } },
      });

      requestCloseTab("tab-orphan-2");

      const closeGuardState = useCloseGuardStore.getState();
      expect(closeGuardState.open).toBe(true);
      expect(closeGuardState.tabIds).toEqual(["tab-orphan-2"]);
    });

    it("closes non-dirty orphaned tab immediately without dialog", () => {
      useWorkspaceStore.setState({
        tabs: [
          {
            id: "tab-orphan-3",
            kind: "query",
            title: "Clean Orphaned Query",
            connectionId: "conn-deleted",
            dirty: false,
            resourceKey: "query:tab-orphan-3",
            data: {
              sql: "SELECT 1;",
              status: "idle",
              error: null,
              result: null,
              explainPlan: null,
              multiResults: null,
              multiResultIndex: 0,
              activeExecutionId: null,
              executionStartedAt: null,
            },
          },
        ],
        activeTabId: "tab-orphan-3",
      });

      requestCloseTab("tab-orphan-3");

      expect(useCloseGuardStore.getState().open).toBe(false);
      expect(useWorkspaceStore.getState().tabs).toHaveLength(0);
    });
  });

  describe("QA-P1-11: Orphan connection reassignment schema normalization", () => {
    it("normalizes schema to 'main' when reassigning a db-object tab from PG to SQLite", () => {
      useWorkspaceStore.setState({
        tabs: [
          {
            id: "tab-reassign-1",
            kind: "db-object",
            title: "users",
            connectionId: "conn-pg",
            resourceKey: "dbobj:public.users:conn-pg",
            dirty: false,
            data: {
              schema: "public",
              objectName: "users",
              objectType: "table",
              activeSection: "data",
            },
          },
        ],
        activeTabId: "tab-reassign-1",
      });

      useWorkspaceStore.getState().reassignTabConnection("tab-reassign-1", "conn-sqlite");

      const updatedTab = useWorkspaceStore.getState().tabs[0];
      expect(updatedTab.connectionId).toBe("conn-sqlite");
      expect(updatedTab.resourceKey).toBe("dbobj:main.users:conn-sqlite");
      if (updatedTab.kind === "db-object") {
        expect(updatedTab.data.schema).toBe("main");
      }
    });

    it("normalizes schema to 'public' when reassigning a db-object tab from SQLite to PG", () => {
      useWorkspaceStore.setState({
        tabs: [
          {
            id: "tab-reassign-2",
            kind: "db-object",
            title: "orders",
            connectionId: "conn-sqlite",
            resourceKey: "dbobj:main.orders:conn-sqlite",
            dirty: false,
            data: {
              schema: "main",
              objectName: "orders",
              objectType: "table",
              activeSection: "data",
            },
          },
        ],
        activeTabId: "tab-reassign-2",
      });

      useWorkspaceStore.getState().reassignTabConnection("tab-reassign-2", "conn-pg");

      const updatedTab = useWorkspaceStore.getState().tabs[0];
      expect(updatedTab.connectionId).toBe("conn-pg");
      expect(updatedTab.resourceKey).toBe("dbobj:public.orders:conn-pg");
      if (updatedTab.kind === "db-object") {
        expect(updatedTab.data.schema).toBe("public");
      }
    });

    it("resets query tab context to default schema for SQLite vs PG", () => {
      useWorkspaceStore.setState({
        tabs: [
          {
            id: "query-reassign",
            kind: "query",
            title: "Query 1",
            connectionId: "conn-pg",
            resourceKey: "query:query-reassign",
            dirty: false,
            data: {
              sql: "SELECT * FROM test",
              context: { database: "postgres", schema: "public" },
              status: "idle",
              error: null,
              result: null,
              explainPlan: null,
              multiResults: null,
              multiResultIndex: 0,
              activeExecutionId: null,
              executionStartedAt: null,
            },
          },
        ],
        activeTabId: "query-reassign",
      });

      useWorkspaceStore.getState().reassignTabConnection("query-reassign", "conn-sqlite");

      const updatedQueryTab = useWorkspaceStore.getState().tabs[0];
      expect(updatedQueryTab.connectionId).toBe("conn-sqlite");
      if (updatedQueryTab.kind === "query") {
        expect(updatedQueryTab.data.context).toEqual({
          database: "/tmp/test.db",
          schema: "main",
        });
      }
    });

    it("clears staged changes on connection reassignment", () => {
      useWorkspaceStore.setState({
        tabs: [
          {
            id: "tab-staged-reassign",
            kind: "db-object",
            title: "products",
            connectionId: "conn-pg",
            resourceKey: "dbobj:public.products:conn-pg",
            dirty: false,
            data: {
              schema: "public",
              objectName: "products",
              objectType: "table",
              activeSection: "data",
            },
          },
        ],
        activeTabId: "tab-staged-reassign",
      });

      useStagedChangesStore.getState().stageCellEdit("tab-staged-reassign", {
        pkValues: [{ type: "int64", value: "10" }],
        changes: { price: { type: "text", value: "120" } },
      });

      expect(useStagedChangesStore.getState().getCount("tab-staged-reassign")).toBe(1);

      useWorkspaceStore.getState().reassignTabConnection("tab-staged-reassign", "conn-sqlite");

      expect(useStagedChangesStore.getState().getCount("tab-staged-reassign")).toBe(0);
    });
  });
});
