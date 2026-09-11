import { afterEach, beforeEach, describe, expect, it } from "vitest";
import type { AnyRouter } from "@tanstack/react-router";

import {
  registerAllCommands,
  resetCommandRegistration,
} from "@/commons/commands/register-commands";
import { createQueryTab } from "@/commons/factories/tab-factories";
import { migratePersistedWorkspace } from "@/commons/services/workspace-migrations";
import { useCommandStore } from "@/commons/stores/command.store";
import { useConnectionStore } from "@/commons/stores/connection.store";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import type { QueryTabData, WorkspaceTab } from "@/commons/types/workspace.types";
import { useTabGridStateStore } from "@/modules/data-grid/state/tab-grid-state.store";
import type { ExplainPlan, QueryResult } from "../types/query.types";

import {
  createExplorerQueryContext,
  createQueryTabForObject,
  createQueryTabFromExplorerContext,
  setQueryTabConnection,
  setQueryTabSchema,
  setTabExplainPlan,
  setTabResult,
  setTabStatus,
} from "../controllers/query-workspace.controller";

const CONNECTIONS = [
  { id: "local", database: "local_db" },
  { id: "production", database: "prod_db" },
  { id: "staging", database: "stage_db" },
];

const SAMPLE_RESULT: QueryResult = {
  columns: [{ name: "id", dataType: "int", nullable: false }],
  rows: [[{ type: "int64", value: "1" }]],
  rowCount: 1,
  durationMs: 5,
};

const SAMPLE_PLAN: ExplainPlan = { "Plan Rows": 1 };

function queryTabById(id: string): (WorkspaceTab & { kind: "query" }) | undefined {
  const tab = useWorkspaceStore.getState().tabs.find((t) => t.id === id);
  return tab?.kind === "query" ? tab : undefined;
}

function resetStores() {
  useWorkspaceStore.setState({ tabs: [], activeTabId: null, recentlyClosed: [] });
  useTabGridStateStore.setState({ states: {} });
  useConnectionStore.setState({
    connections: [],
    explorerConnectionId: null,
    isLoading: false,
    error: null,
  });
  useCommandStore.setState({ commands: [], isOpen: false });
}

describe("per-tab query context (UX-R7.1)", () => {
  beforeEach(resetStores);
  afterEach(resetStores);

  describe("new query inherits explorer context", () => {
    it("builds explorer context from the explorer connection", () => {
      const context = createExplorerQueryContext(CONNECTIONS, "local");
      expect(context).toEqual({ database: "local_db", schema: null });
      expect(createExplorerQueryContext(CONNECTIONS, null)).toEqual({
        database: null,
        schema: null,
      });
    });

    it("creates a query tab from the explorer context primitive", () => {
      useConnectionStore.setState({ connections: CONNECTIONS, explorerConnectionId: "local" });

      const tab = createQueryTabFromExplorerContext("local");

      expect(tab).toBeDefined();
      expect(tab?.connectionId).toBe("local");
      expect(tab?.data.context).toEqual({ database: "local_db", schema: null });
    });

    it("returns undefined without an explorer connection", () => {
      const tab = createQueryTabFromExplorerContext(null);
      expect(tab).toBeUndefined();
    });

    it("defaults to unbound context when no context is provided", () => {
      const tab = createQueryTab("local");
      expect(tab.data.context).toEqual({ database: null, schema: null });
    });
  });

  describe("tabs.new command integration", () => {
    it("opens a query tab that inherits the explorer context", () => {
      useConnectionStore.setState({ connections: CONNECTIONS, explorerConnectionId: "local" });
      resetCommandRegistration();
      registerAllCommands({} as AnyRouter);

      useCommandStore.getState().executeCommand("tabs.new");

      const state = useWorkspaceStore.getState();
      expect(state.tabs).toHaveLength(1);
      expect(state.activeTabId).toBe(state.tabs[0].id);
      const opened = queryTabById(state.tabs[0].id);
      expect(opened?.connectionId).toBe("local");
      expect(opened?.data.context).toEqual({ database: "local_db", schema: null });
    });

    it("nav.query reuses an existing tab instead of duplicating", () => {
      useConnectionStore.setState({ connections: CONNECTIONS, explorerConnectionId: "local" });
      resetCommandRegistration();
      registerAllCommands({} as AnyRouter);

      useCommandStore.getState().executeCommand("tabs.new");
      const firstId = useWorkspaceStore.getState().tabs[0].id;

      useCommandStore.getState().executeCommand("nav.query");

      const state = useWorkspaceStore.getState();
      expect(state.tabs).toHaveLength(1);
      expect(state.activeTabId).toBe(firstId);
    });
  });

  describe("tab context independence", () => {
    it("keeps two query tab contexts independent", () => {
      const tabA = createQueryTab("production", {
        title: "Query A",
        context: { database: "prod_db", schema: "audit" },
      });
      const tabB = createQueryTab("local", {
        title: "Query B",
        context: { database: "local_db", schema: "auth" },
      });
      const { openTab } = useWorkspaceStore.getState();
      openTab(tabA);
      openTab(tabB);

      setQueryTabSchema(tabA.id, "reporting");

      const a = queryTabById(tabA.id);
      const b = queryTabById(tabB.id);
      expect(a?.data.context).toEqual({ database: "prod_db", schema: "reporting" });
      expect(b?.data.context).toEqual({ database: "local_db", schema: "auth" });
    });

    it("does not mutate query tabs when the explorer connection changes", () => {
      const tabA = createQueryTab("production", {
        context: { database: "prod_db", schema: "audit" },
      });
      const tabB = createQueryTab("local", {
        context: { database: "local_db", schema: "auth" },
      });
      const { openTab } = useWorkspaceStore.getState();
      openTab(tabA);
      openTab(tabB);

      useConnectionStore.getState().setExplorerConnection("staging");

      const a = queryTabById(tabA.id);
      const b = queryTabById(tabB.id);
      expect(a?.connectionId).toBe("production");
      expect(a?.data.context).toEqual({ database: "prod_db", schema: "audit" });
      expect(b?.connectionId).toBe("local");
      expect(b?.data.context).toEqual({ database: "local_db", schema: "auth" });
    });
  });

  describe("object → Open SELECT", () => {
    it("resolves connection, database and schema through the real creation path", () => {
      useConnectionStore.setState({ connections: CONNECTIONS, explorerConnectionId: "local" });

      const tab = createQueryTabForObject("production", "public", {
        title: "SELECT users",
        sql: 'SELECT * FROM "public"."users" LIMIT 100;',
      });

      expect(tab.connectionId).toBe("production");
      expect(tab.data.context).toEqual({ database: "prod_db", schema: "public" });
      expect(tab.data.sql).toContain('"users"');
    });

    it("keeps database null when the connection is not yet known", () => {
      useConnectionStore.setState({ connections: [], explorerConnectionId: null });

      const tab = createQueryTabForObject("production", "public", { sql: "SELECT 1" });

      expect(tab.connectionId).toBe("production");
      expect(tab.data.context).toEqual({ database: null, schema: "public" });
    });
  });

  describe("persisted workspace migration", () => {
    it("migrates legacy query tabs without context safely", () => {
      const legacyTab = createQueryTab("local", { title: "Q1", sql: "SELECT 1" });
      delete (legacyTab.data as Partial<QueryTabData>).context;
      const tabs = [legacyTab];

      const migrated = migratePersistedWorkspace(tabs);

      expect(migrated).not.toBe(tabs);
      expect(migrated[0].kind).toBe("query");
      expect((migrated[0].data as QueryTabData).context).toEqual({
        database: null,
        schema: null,
      });
      expect((migrated[0].data as QueryTabData).sql).toBe("SELECT 1");
    });

    it("leaves tabs that already carry context untouched", () => {
      const freshTab = createQueryTab("local", {
        context: { database: "local_db", schema: "auth" },
      });
      const tabs = [freshTab];
      const result = migratePersistedWorkspace(tabs);
      expect(result).toBe(tabs);
    });
  });

  describe("reopen closed tab", () => {
    it("preserves context when a closed tab is reopened", () => {
      const tab = createQueryTab("production", {
        title: "Q1",
        sql: "SELECT 1",
        context: { database: "prod_db", schema: "audit" },
      });
      const { openTab, closeTab, reopenLastClosed } = useWorkspaceStore.getState();
      openTab(tab);
      closeTab(tab.id);
      reopenLastClosed();

      const reopened = queryTabById(tab.id);
      expect(reopened?.connectionId).toBe("production");
      expect(reopened?.data.context).toEqual({ database: "prod_db", schema: "audit" });
      expect(reopened?.data.sql).toBe("SELECT 1");
    });
  });

  describe("connection switch contract", () => {
    it("clears stale execution artifacts but keeps SQL", () => {
      const tab = createQueryTab("production", {
        title: "Q1",
        sql: "SELECT * FROM audit.logs",
        context: { database: "prod_db", schema: "audit" },
      });
      const { openTab } = useWorkspaceStore.getState();
      openTab(tab);

      setTabStatus(tab.id, "success");
      setTabResult(tab.id, SAMPLE_RESULT);
      setTabExplainPlan(tab.id, SAMPLE_PLAN);

      setQueryTabConnection(tab.id, "staging", { database: "stage_db", schema: null });

      const current = queryTabById(tab.id);
      expect(current?.connectionId).toBe("staging");
      expect(current?.data.sql).toBe("SELECT * FROM audit.logs");
      expect(current?.data.context).toEqual({ database: "stage_db", schema: null });
      expect(current?.data.status).toBe("idle");
      expect(current?.data.error).toBeNull();
      expect(current?.data.result).toBeNull();
      expect(current?.data.explainPlan).toBeNull();
      expect(current?.data.multiResults).toBeNull();
    });

    it("keeps SQL on schema change and does not affect other tabs", () => {
      const tabA = createQueryTab("production", {
        title: "A",
        sql: "SELECT 1",
        context: { database: "prod_db", schema: "audit" },
      });
      const tabB = createQueryTab("production", {
        title: "B",
        sql: "SELECT 2",
        context: { database: "prod_db", schema: "public" },
      });
      const { openTab } = useWorkspaceStore.getState();
      openTab(tabA);
      openTab(tabB);

      setQueryTabSchema(tabA.id, "reporting");

      const a = queryTabById(tabA.id);
      const b = queryTabById(tabB.id);
      expect(a?.data.sql).toBe("SELECT 1");
      expect(a?.data.context).toEqual({ database: "prod_db", schema: "reporting" });
      expect(b?.data.sql).toBe("SELECT 2");
      expect(b?.data.context).toEqual({ database: "prod_db", schema: "public" });
    });
  });
});
