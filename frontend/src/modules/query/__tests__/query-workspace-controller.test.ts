import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { createQueryTab } from "@/commons/factories/tab-factories";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import type { QueryTiming } from "@/commons/types/workspace.types";
import {
  getActiveQueryTab,
  getQueryContext,
  getQueryTabData,
  buildQueryContext,
  createExplorerQueryContext,
  setTabSql,
  setTabStatus,
  setTabError,
  setTabResult,
  setTabExplainPlan,
  setTabSort,
  setTabMultiResults,
  setTabMultiResultIndex,
  setTabActivePanel,
  setTabTiming,
  setTabExecutionStartedAt,
  setQueryTabSchema,
} from "../controllers/query-workspace.controller";

function resetStore() {
  useWorkspaceStore.setState({
    tabs: [],
    activeTabId: null,
    recentlyClosed: [],
  });
}

describe("QueryWorkspaceController", () => {
  beforeEach(resetStore);
  afterEach(resetStore);

  function openQueryTab(opts?: { sql?: string; title?: string }) {
    const tab = createQueryTab("conn-1", { title: opts?.title ?? "Q1", sql: opts?.sql ?? "SELECT 1" });
    useWorkspaceStore.getState().openTab(tab);
    return tab;
  }

  describe("getActiveQueryTab", () => {
    it("returns the active query tab", () => {
      const tab = openQueryTab();
      const active = getActiveQueryTab();
      expect(active).toBeDefined();
      expect(active!.id).toBe(tab.id);
      expect(active!.kind).toBe("query");
    });

    it("returns undefined when no active tab", () => {
      expect(getActiveQueryTab()).toBeUndefined();
    });

    it("returns undefined when active tab is not a query tab", () => {
      openQueryTab();
      useWorkspaceStore.setState({ activeTabId: null });
      expect(getActiveQueryTab()).toBeUndefined();
    });
  });

  describe("getQueryTabData", () => {
    it("returns QueryTabData for a query tab", () => {
      const tab = openQueryTab({ sql: "SELECT 42" });
      const data = getQueryTabData(tab.id);
      expect(data).toBeDefined();
      expect(data!.sql).toBe("SELECT 42");
    });

    it("returns undefined for unknown tab id", () => {
      expect(getQueryTabData("nonexistent")).toBeUndefined();
    });
  });

  describe("getQueryContext", () => {
    it("returns context from query tab data", () => {
      const tab = openQueryTab();
      const ctx = getQueryContext(tab.id);
      expect(ctx).toBeDefined();
    });

    it("returns undefined for unknown tab", () => {
      expect(getQueryContext("nonexistent")).toBeUndefined();
    });
  });

  describe("buildQueryContext", () => {
    it("resolves database from connection id", () => {
      const connections = [
        { id: "conn-1", database: "mydb" },
        { id: "conn-2", database: "otherdb" },
      ];
      const ctx = buildQueryContext(connections, "conn-1", "public");
      expect(ctx.database).toBe("mydb");
      expect(ctx.schema).toBe("public");
    });

    it("returns null database for unknown connection", () => {
      const ctx = buildQueryContext([], "conn-unknown", "public");
      expect(ctx.database).toBeNull();
    });

    it("handles null connectionId", () => {
      const ctx = buildQueryContext([{ id: "c1", database: "db" }], null, null);
      expect(ctx.database).toBeNull();
      expect(ctx.schema).toBeNull();
    });
  });

  describe("createExplorerQueryContext", () => {
    it("builds context from explorer connection id", () => {
      const connections = [{ id: "conn-1", database: "testdb" }];
      const ctx = createExplorerQueryContext(connections, "conn-1");
      expect(ctx.database).toBe("testdb");
      expect(ctx.schema).toBeNull();
    });

    it("returns null database when no explorer connection", () => {
      const ctx = createExplorerQueryContext([], null);
      expect(ctx.database).toBeNull();
    });
  });

  describe("setTabSql", () => {
    it("updates SQL and marks tab dirty", () => {
      const tab = openQueryTab({ sql: "SELECT 1" });
      setTabSql(tab.id, "SELECT 2");
      const data = getQueryTabData(tab.id);
      expect(data!.sql).toBe("SELECT 2");
      const wsTab = useWorkspaceStore.getState().tabs.find((t) => t.id === tab.id);
      expect(wsTab!.dirty).toBe(true);
    });

    it("does not set dirty flag if already dirty", () => {
      const tab = openQueryTab();
      useWorkspaceStore.getState().setTabDirty(tab.id, true);
      setTabSql(tab.id, "SELECT 99");
      const wsTab = useWorkspaceStore.getState().tabs.find((t) => t.id === tab.id);
      expect(wsTab!.dirty).toBe(true);
    });
  });

  describe("setTabStatus", () => {
    it("updates execution status", () => {
      const tab = openQueryTab();
      setTabStatus(tab.id, "running");
      expect(getQueryTabData(tab.id)!.status).toBe("running");
      setTabStatus(tab.id, "success");
      expect(getQueryTabData(tab.id)!.status).toBe("success");
    });
  });

  describe("setTabError", () => {
    it("sets and clears error", () => {
      const tab = openQueryTab();
      setTabError(tab.id, "Something went wrong");
      expect(getQueryTabData(tab.id)!.error).toBe("Something went wrong");
      setTabError(tab.id, null);
      expect(getQueryTabData(tab.id)!.error).toBeNull();
    });
  });

  describe("setTabResult", () => {
    it("sets query result", () => {
      const tab = openQueryTab();
      const result = { columns: ["id"], rows: [[1]], rowCount: 1, durationMs: 5 };
      setTabResult(tab.id, result);
      expect(getQueryTabData(tab.id)!.result).toEqual(result);
    });

    it("clears result with null", () => {
      const tab = openQueryTab();
      setTabResult(tab.id, null);
      expect(getQueryTabData(tab.id)!.result).toBeNull();
    });
  });

  describe("setTabExplainPlan", () => {
    it("sets explain plan", () => {
      const tab = openQueryTab();
      const plan = { Plan: { "Node Type": "Seq Scan" } };
      setTabExplainPlan(tab.id, plan);
      expect(getQueryTabData(tab.id)!.explainPlan).toEqual(plan);
    });
  });

  describe("setTabSort", () => {
    it("sets sort state", () => {
      const tab = openQueryTab();
      setTabSort(tab.id, { column: "name", direction: "asc" });
      expect(getQueryTabData(tab.id)!.sort).toEqual({ column: "name", direction: "asc" });
    });
  });

  describe("setTabMultiResults", () => {
    it("sets multi results and resets index to 0", () => {
      const tab = openQueryTab();
      const results = [
        { columns: ["a"], rows: [[1]], rowCount: 1, durationMs: 1 },
        { columns: ["b"], rows: [[2]], rowCount: 1, durationMs: 2 },
      ];
      setTabMultiResults(tab.id, results);
      const data = getQueryTabData(tab.id)!;
      expect(data.multiResults).toEqual(results);
      expect(data.multiResultIndex).toBe(0);
    });
  });

  describe("setTabMultiResultIndex", () => {
    it("sets multi result index", () => {
      const tab = openQueryTab();
      setTabMultiResultIndex(tab.id, 2);
      expect(getQueryTabData(tab.id)!.multiResultIndex).toBe(2);
    });
  });

  describe("setTabActivePanel", () => {
    it("sets active panel tab", () => {
      const tab = openQueryTab();
      setTabActivePanel(tab.id, "explain");
      expect(getQueryTabData(tab.id)!.activePanel).toBe("explain");
    });
  });

  describe("setTabTiming", () => {
    it("sets timing information", () => {
      const tab = openQueryTab();
      const timing: QueryTiming = { serverMs: 50, totalMs: 120, fetchMs: 60, renderMs: 10 };
      setTabTiming(tab.id, timing);
      expect(getQueryTabData(tab.id)!.timing).toEqual(timing);
    });

    it("clears timing with null", () => {
      const tab = openQueryTab();
      setTabTiming(tab.id, null);
      expect(getQueryTabData(tab.id)!.timing).toBeNull();
    });
  });

  describe("setTabExecutionStartedAt", () => {
    it("sets execution start timestamp", () => {
      const tab = openQueryTab();
      const now = Date.now();
      setTabExecutionStartedAt(tab.id, now);
      expect(getQueryTabData(tab.id)!.executionStartedAt).toBe(now);
    });

    it("clears with null", () => {
      const tab = openQueryTab();
      setTabExecutionStartedAt(tab.id, null);
      expect(getQueryTabData(tab.id)!.executionStartedAt).toBeNull();
    });
  });

  describe("setQueryTabSchema", () => {
    it("updates schema in context", () => {
      const tab = openQueryTab();
      setQueryTabSchema(tab.id, "audit");
      const data = getQueryTabData(tab.id)!;
      expect(data.context.schema).toBe("audit");
    });

    it("sets schema to null", () => {
      const tab = openQueryTab();
      setQueryTabSchema(tab.id, null);
      const data = getQueryTabData(tab.id)!;
      expect(data.context.schema).toBeNull();
    });
  });
});
