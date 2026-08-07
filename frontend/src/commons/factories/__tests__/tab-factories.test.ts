import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { createQueryTab, createDbObjectTab } from "../tab-factories";

function resetStore() {
  useWorkspaceStore.setState({ tabs: [], activeTabId: null, recentlyClosed: [] });
}

describe("tab-factories", () => {
  beforeEach(resetStore);
  afterEach(resetStore);

  describe("createQueryTab", () => {
    it("creates a query tab with defaults", () => {
      const tab = createQueryTab("conn-1");
      expect(tab.kind).toBe("query");
      expect(tab.connectionId).toBe("conn-1");
      expect(tab.dirty).toBe(false);
      expect(tab.pinned).toBe(false);
      expect(tab.preview).toBe(false);
      expect(tab.data.sql).toBe("");
      expect(tab.data.status).toBe("idle");
      expect(tab.data.error).toBeNull();
      expect(tab.data.result).toBeNull();
      expect(tab.data.timing).toBeNull();
      expect(tab.data.executionStartedAt).toBeNull();
    });

    it("creates a query tab with custom options", () => {
      const tab = createQueryTab("conn-1", {
        title: "My Query",
        sql: "SELECT 1",
        context: { database: "mydb", schema: "public" },
      });
      expect(tab.title).toBe("My Query");
      expect(tab.data.sql).toBe("SELECT 1");
      expect(tab.data.context.database).toBe("mydb");
      expect(tab.data.context.schema).toBe("public");
    });

    it("auto-generates title when not provided", () => {
      const tab = createQueryTab("conn-1");
      expect(tab.title).toMatch(/^Query \d+$/);
    });

    it("generates sequential titles for same connection", () => {
      const tab1 = createQueryTab("conn-1");
      useWorkspaceStore.getState().openTab(tab1);
      const tab2 = createQueryTab("conn-1");
      expect(tab1.title).not.toBe(tab2.title);
    });

    it("generates unique resourceKey", () => {
      const tab1 = createQueryTab("conn-1");
      const tab2 = createQueryTab("conn-1");
      expect(tab1.resourceKey).not.toBe(tab2.resourceKey);
    });

    it("sets order based on existing tabs", () => {
      const tab1 = createQueryTab("conn-1");
      useWorkspaceStore.getState().openTab(tab1);
      const tab2 = createQueryTab("conn-1");
      expect(tab2.order).toBeGreaterThan(tab1.order);
    });
  });

  describe("createDbObjectTab", () => {
    it("creates a db-object tab with correct data", () => {
      const tab = createDbObjectTab("conn-1", "public", "users", "table");
      expect(tab.kind).toBe("db-object");
      expect(tab.connectionId).toBe("conn-1");
      expect(tab.title).toBe("users");
      expect(tab.data.schema).toBe("public");
      expect(tab.data.objectName).toBe("users");
      expect(tab.data.objectType).toBe("table");
      expect(tab.data.activeSection).toBe("columns");
    });

    it("creates a view tab", () => {
      const tab = createDbObjectTab("conn-1", "public", "v_active", "view", "data", false);
      expect(tab.data.objectType).toBe("view");
      expect(tab.data.activeSection).toBe("data");
      expect(tab.preview).toBe(false);
    });

    it("generates correct resourceKey", () => {
      const tab = createDbObjectTab("conn-1", "public", "users", "table");
      expect(tab.resourceKey).toBe("dbobj:public.users:conn-1");
    });

    it("defaults to preview mode", () => {
      const tab = createDbObjectTab("conn-1", "public", "users", "table");
      expect(tab.preview).toBe(true);
    });

    it("supports non-preview mode", () => {
      const tab = createDbObjectTab("conn-1", "public", "users", "table", "columns", false);
      expect(tab.preview).toBe(false);
    });

    it("supports different initial sections", () => {
      const tab = createDbObjectTab("conn-1", "public", "users", "table", "indexes");
      expect(tab.data.activeSection).toBe("indexes");
    });
  });
});
