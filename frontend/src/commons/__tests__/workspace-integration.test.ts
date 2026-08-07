import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { createQueryTab, createDbObjectTab } from "@/commons/factories/tab-factories";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { useTabGridStateStore } from "@/modules/data-grid/state/tab-grid-state.store";
import type { QueryTabData } from "@/commons/types/workspace.types";

/**
 * Integration tests that exercise multi-step workspace flows
 * combining tab lifecycle, data updates, and grid state.
 */

function resetStores() {
  useWorkspaceStore.setState({ tabs: [], activeTabId: null, recentlyClosed: [] });
  useTabGridStateStore.setState({ states: {} });
}

describe("Workspace integration flows", () => {
  beforeEach(resetStores);
  afterEach(resetStores);

  describe("query tab lifecycle", () => {
    it("create → edit → execute → close → reopen preserves SQL", () => {
      const { openTab, closeTab, updateTabData, reopenLastClosed, setTabDirty } = useWorkspaceStore.getState();

      // 1. Create query tab
      const tab = createQueryTab("conn-1", { title: "My Query", sql: "SELECT 1" });
      openTab(tab);
      expect(useWorkspaceStore.getState().tabs).toHaveLength(1);

      // 2. Edit SQL
      updateTabData(tab.id, (d: QueryTabData) => ({ ...d, sql: "SELECT * FROM users WHERE id = 1" }));
      setTabDirty(tab.id, true);

      // 3. Simulate execution result
      updateTabData(tab.id, (d: QueryTabData) => ({
        ...d,
        status: "success",
        result: { columns: ["id", "name"], rows: [[1, "Alice"]], rowCount: 1, durationMs: 12 },
      }));

      // 4. Close tab
      closeTab(tab.id);
      expect(useWorkspaceStore.getState().tabs).toHaveLength(0);
      expect(useWorkspaceStore.getState().recentlyClosed).toHaveLength(1);

      // 5. Reopen
      reopenLastClosed();
      const state = useWorkspaceStore.getState();
      expect(state.tabs).toHaveLength(1);
      const reopened = state.tabs[0];
      expect(reopened.kind).toBe("query");
      if (reopened.kind === "query") {
        expect(reopened.data.sql).toBe("SELECT * FROM users WHERE id = 1");
        expect(reopened.data.result).toBeDefined();
        expect(reopened.data.result!.rowCount).toBe(1);
      }
    });
  });

  describe("multi-tab query isolation", () => {
    it("updating one tab's data does not affect other tabs", () => {
      const { openTab, updateTabData } = useWorkspaceStore.getState();

      const tab1 = createQueryTab("conn-1", { title: "Q1", sql: "SELECT 1" });
      const tab2 = createQueryTab("conn-1", { title: "Q2", sql: "SELECT 2" });
      openTab(tab1);
      openTab(tab2);

      updateTabData(tab1.id, (d: QueryTabData) => ({
        ...d,
        status: "running",
        sql: "SELECT * FROM large_table",
      }));

      const state = useWorkspaceStore.getState();
      const t1 = state.tabs.find((t) => t.id === tab1.id);
      const t2 = state.tabs.find((t) => t.id === tab2.id);

      expect(t1!.kind).toBe("query");
      if (t1!.kind === "query") {
        expect(t1.data.status).toBe("running");
        expect(t1.data.sql).toBe("SELECT * FROM large_table");
      }

      expect(t2!.kind).toBe("query");
      if (t2!.kind === "query") {
        expect(t2.data.status).toBe("idle");
        expect(t2.data.sql).toBe("SELECT 2");
      }
    });
  });

  describe("db-object tab with grid state", () => {
    it("open table → set grid state → close → grid state cleaned", () => {
      const { openDbObject } = useWorkspaceStore.getState();

      const tab = createDbObjectTab("conn-1", "public", "users", "table", "columns", false);
      openDbObject(tab);

      // Set grid state for this tab
      useTabGridStateStore.getState().setState(tab.id, {
        page: 5,
        filters: [{ column: "status", operator: "eq", value: "active", enabled: true }],
        sorts: [{ column: "name", direction: "asc" }],
      });

      expect(useTabGridStateStore.getState().getState(tab.id).page).toBe(5);

      // Close the tab
      useWorkspaceStore.getState().closeTab(tab.id);

      // GC grid state
      const validIds = new Set(useWorkspaceStore.getState().tabs.map((t) => t.id));
      useTabGridStateStore.getState().gc(validIds);

      // Grid state should be gone
      const gridState = useTabGridStateStore.getState().getState(tab.id);
      expect(gridState.page).toBe(1); // defaults
      expect(gridState.filters).toHaveLength(0);
    });
  });

  describe("mixed tab types", () => {
    it("manages query and db-object tabs side by side", () => {
      const { openTab, openDbObject } = useWorkspaceStore.getState();

      const queryTab = createQueryTab("conn-1", { title: "Q1" });
      openTab(queryTab);

      const dbTab = createDbObjectTab("conn-1", "public", "orders", "table", "data", false);
      openDbObject(dbTab);

      const state = useWorkspaceStore.getState();
      expect(state.tabs).toHaveLength(2);
      expect(state.activeTabId).toBe(dbTab.id);

      // Close query tab
      useWorkspaceStore.getState().closeTab(queryTab.id);
      expect(useWorkspaceStore.getState().tabs).toHaveLength(1);
      expect(useWorkspaceStore.getState().tabs[0].kind).toBe("db-object");
    });
  });

  describe("workspace state restore", () => {
    it("restores complex workspace with multiple tab types", () => {
      const { openTab, openDbObject, restoreState } = useWorkspaceStore.getState();

      const q1 = createQueryTab("conn-1", { title: "Q1", sql: "SELECT 1" });
      const q2 = createQueryTab("conn-1", { title: "Q2", sql: "SELECT 2" });
      const db1 = createDbObjectTab("conn-1", "public", "users", "table", "columns", false);

      openTab(q1);
      openTab(q2);
      openDbObject(db1);

      // Capture state
      const captured = {
        tabs: useWorkspaceStore.getState().tabs,
        activeTabId: useWorkspaceStore.getState().activeTabId,
        recentlyClosed: useWorkspaceStore.getState().recentlyClosed,
      };

      // Reset
      resetStores();
      expect(useWorkspaceStore.getState().tabs).toHaveLength(0);

      // Restore
      restoreState(captured);
      const state = useWorkspaceStore.getState();
      expect(state.tabs).toHaveLength(3);
      expect(state.activeTabId).toBe(captured.activeTabId);
    });
  });

  describe("tab ordering and pinning", () => {
    it("pinned tabs stay in place during close operations", () => {
      const { openTab, toggleTabPinned, closeOthers } = useWorkspaceStore.getState();

      const q1 = createQueryTab("conn-1", { title: "Q1" });
      const q2 = createQueryTab("conn-1", { title: "Q2" });
      const q3 = createQueryTab("conn-1", { title: "Q3" });

      openTab(q1);
      openTab(q2);
      openTab(q3);
      toggleTabPinned(q1.id);

      closeOthers(q2.id);

      const state = useWorkspaceStore.getState();
      expect(state.tabs).toHaveLength(2);
      const ids = state.tabs.map((t) => t.id);
      expect(ids).toContain(q1.id); // pinned
      expect(ids).toContain(q2.id); // target
      expect(ids).not.toContain(q3.id); // closed
    });
  });

  describe("preview tab lifecycle", () => {
    it("preview → promote → edit → close cycle", () => {
      const { openDbObject, promotePreview, setDbObjectSection } = useWorkspaceStore.getState();

      // Open preview
      const preview = createDbObjectTab("conn-1", "public", "users", "table", "columns", true);
      openDbObject(preview);
      expect(useWorkspaceStore.getState().tabs[0].preview).toBe(true);

      // Promote
      promotePreview(preview.id);
      expect(useWorkspaceStore.getState().tabs[0].preview).toBe(false);

      // Switch section
      setDbObjectSection(preview.id, "data");
      if (useWorkspaceStore.getState().tabs[0].kind === "db-object") {
        expect(useWorkspaceStore.getState().tabs[0].data.activeSection).toBe("data");
      }

      // Close
      useWorkspaceStore.getState().closeTab(preview.id);
      expect(useWorkspaceStore.getState().tabs).toHaveLength(0);
    });
  });
});
