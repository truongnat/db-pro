import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { createQueryTab, createDbObjectTab } from "@/commons/factories/tab-factories";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { useTabGridStateStore } from "@/modules/data-grid/state/tab-grid-state.store";

function resetStore() {
  useWorkspaceStore.setState({
    tabs: [],
    activeTabId: null,
    recentlyClosed: [],
  });
  useTabGridStateStore.setState({ states: {} });
}

describe("WorkspaceStore", () => {
  beforeEach(resetStore);
  afterEach(resetStore);

  describe("openTab", () => {
    it("adds a new tab and activates it", () => {
      const tab = createQueryTab("conn-1", "Query 1");
      useWorkspaceStore.getState().openTab(tab);

      const state = useWorkspaceStore.getState();
      expect(state.tabs).toHaveLength(1);
      expect(state.activeTabId).toBe(tab.id);
    });

    it("deduplicates by resourceKey — activates existing tab", () => {
      const tab1 = createDbObjectTab("conn-1", "public", "users", "table");
      const tab2 = createDbObjectTab("conn-1", "public", "users", "table");

      useWorkspaceStore.getState().openTab(tab1);
      useWorkspaceStore.getState().openTab(tab2);

      const state = useWorkspaceStore.getState();
      expect(state.tabs).toHaveLength(1);
      expect(state.activeTabId).toBe(tab1.id);
    });

    it("replaces preview tab when opening same kind", () => {
      const tab1 = createQueryTab("conn-1", "Preview 1");
      tab1.preview = true;
      useWorkspaceStore.getState().openTab(tab1);

      const tab2 = createQueryTab("conn-1", "Preview 2");
      tab2.preview = true;
      useWorkspaceStore.getState().openTab(tab2);

      const state = useWorkspaceStore.getState();
      expect(state.tabs).toHaveLength(1);
      expect(state.tabs[0].title).toBe("Preview 2");
      expect(state.tabs[0].id).toBe(tab1.id);
    });

    it("promotes preview tab when non-preview opens with same resourceKey", () => {
      const tab = createQueryTab("conn-1", "Preview");
      tab.preview = true;
      useWorkspaceStore.getState().openTab(tab);

      const same = { ...tab, preview: false };
      useWorkspaceStore.getState().openTab(same);

      const state = useWorkspaceStore.getState();
      expect(state.tabs).toHaveLength(1);
      expect(state.tabs[0].preview).toBe(false);
    });
  });

  describe("closeTab", () => {
    it("closes tab and activates neighbor", () => {
      const tab1 = createQueryTab("conn-1", "Q1");
      const tab2 = createQueryTab("conn-1", "Q2");
      const tab3 = createQueryTab("conn-1", "Q3");
      const { openTab, closeTab } = useWorkspaceStore.getState();

      openTab(tab1);
      openTab(tab2);
      openTab(tab3);

      closeTab(tab2.id);

      const state = useWorkspaceStore.getState();
      expect(state.tabs).toHaveLength(2);
      expect(state.activeTabId).toBe(tab3.id);
    });

    it("moves closed tab to recentlyClosed", () => {
      const tab1 = createQueryTab("conn-1", "Q1");
      const tab2 = createQueryTab("conn-1", "Q2");
      const { openTab, closeTab } = useWorkspaceStore.getState();

      openTab(tab1);
      openTab(tab2);
      closeTab(tab1.id);

      const state = useWorkspaceStore.getState();
      expect(state.recentlyClosed).toHaveLength(1);
      expect(state.recentlyClosed[0].id).toBe(tab1.id);
    });

    it("activates previous tab when closing the last tab", () => {
      const tab1 = createQueryTab("conn-1", "Q1");
      const tab2 = createQueryTab("conn-1", "Q2");
      const { openTab, closeTab } = useWorkspaceStore.getState();

      openTab(tab1);
      openTab(tab2);

      closeTab(tab2.id);

      const state = useWorkspaceStore.getState();
      expect(state.activeTabId).toBe(tab1.id);
    });

    it("allows closing the last tab", () => {
      const tab = createQueryTab("conn-1", "Q1");
      const { openTab, closeTab } = useWorkspaceStore.getState();

      openTab(tab);
      closeTab(tab.id);

      const state = useWorkspaceStore.getState();
      expect(state.tabs).toHaveLength(0);
      expect(state.activeTabId).toBeNull();
    });
  });

  describe("reopenLastClosed", () => {
    it("reopens the last closed tab preserving original id and resourceKey", () => {
      const tab1 = createQueryTab("conn-1", "Q1");
      const tab2 = createQueryTab("conn-1", "Q2");
      const { openTab, closeTab, reopenLastClosed } = useWorkspaceStore.getState();

      openTab(tab1);
      openTab(tab2);
      closeTab(tab1.id);

      reopenLastClosed();

      const state = useWorkspaceStore.getState();
      expect(state.tabs).toHaveLength(2);
      expect(state.recentlyClosed).toHaveLength(0);
      const reopened = state.tabs.find((t) => t.title === "Q1");
      expect(reopened).toBeDefined();
      expect(reopened!.id).toBe(tab1.id);
      expect(reopened!.resourceKey).toBe(tab1.resourceKey);
    });

    it("does nothing when recentlyClosed is empty", () => {
      const tab = createQueryTab("conn-1", "Q1");
      useWorkspaceStore.getState().openTab(tab);

      useWorkspaceStore.getState().reopenLastClosed();

      const state = useWorkspaceStore.getState();
      expect(state.tabs).toHaveLength(1);
    });
  });

  describe("closeOthers / closeRight", () => {
    it("closeOthers keeps only target + pinned tabs", () => {
      const tab1 = createQueryTab("conn-1", "Q1");
      const tab2 = createQueryTab("conn-1", "Q2");
      const tab3 = createQueryTab("conn-1", "Q3");
      const { openTab, toggleTabPinned, closeOthers } = useWorkspaceStore.getState();

      openTab(tab1);
      openTab(tab2);
      openTab(tab3);
      toggleTabPinned(tab3.id);
      closeOthers(tab2.id);

      const state = useWorkspaceStore.getState();
      expect(state.tabs).toHaveLength(2);
      expect(state.tabs.map((t) => t.id)).toContain(tab2.id);
      expect(state.tabs.map((t) => t.id)).toContain(tab3.id);
    });

    it("closeRight keeps tabs up to and including target + pinned", () => {
      const tab1 = createQueryTab("conn-1", "Q1");
      const tab2 = createQueryTab("conn-1", "Q2");
      const tab3 = createQueryTab("conn-1", "Q3");
      const { openTab, closeRight } = useWorkspaceStore.getState();

      openTab(tab1);
      openTab(tab2);
      openTab(tab3);
      closeRight(tab1.id);

      const state = useWorkspaceStore.getState();
      expect(state.tabs).toHaveLength(1);
      expect(state.tabs[0].id).toBe(tab1.id);
    });

    it("closeOthers pushes evicted tabs to recentlyClosed", () => {
      const tab1 = createQueryTab("conn-1", "Q1");
      const tab2 = createQueryTab("conn-1", "Q2");
      const tab3 = createQueryTab("conn-1", "Q3");
      const { openTab, closeOthers } = useWorkspaceStore.getState();

      openTab(tab1);
      openTab(tab2);
      openTab(tab3);
      closeOthers(tab3.id);

      const state = useWorkspaceStore.getState();
      expect(state.recentlyClosed).toHaveLength(2);
      expect(state.recentlyClosed.map((t) => t.id)).toContain(tab1.id);
      expect(state.recentlyClosed.map((t) => t.id)).toContain(tab2.id);
    });

    it("closeRight pushes evicted tabs to recentlyClosed", () => {
      const tab1 = createQueryTab("conn-1", "Q1");
      const tab2 = createQueryTab("conn-1", "Q2");
      const tab3 = createQueryTab("conn-1", "Q3");
      const { openTab, closeRight } = useWorkspaceStore.getState();

      openTab(tab1);
      openTab(tab2);
      openTab(tab3);
      closeRight(tab1.id);

      const state = useWorkspaceStore.getState();
      expect(state.recentlyClosed).toHaveLength(2);
      expect(state.recentlyClosed.map((t) => t.id)).toContain(tab2.id);
      expect(state.recentlyClosed.map((t) => t.id)).toContain(tab3.id);
    });
  });

  describe("closeTabs", () => {
    it("batch closes multiple tabs and populates recentlyClosed", () => {
      const tab1 = createQueryTab("conn-1", "Q1");
      const tab2 = createQueryTab("conn-1", "Q2");
      const tab3 = createQueryTab("conn-1", "Q3");
      const { openTab, closeTabs } = useWorkspaceStore.getState();

      openTab(tab1);
      openTab(tab2);
      openTab(tab3);
      closeTabs([tab1.id, tab2.id]);

      const state = useWorkspaceStore.getState();
      expect(state.tabs).toHaveLength(1);
      expect(state.tabs[0].id).toBe(tab3.id);
      expect(state.recentlyClosed).toHaveLength(2);
    });

    it("skips pinned tabs in batch close", () => {
      const tab1 = createQueryTab("conn-1", "Q1");
      const tab2 = createQueryTab("conn-1", "Q2");
      const { openTab, toggleTabPinned, closeTabs } = useWorkspaceStore.getState();

      openTab(tab1);
      openTab(tab2);
      toggleTabPinned(tab1.id);
      closeTabs([tab1.id, tab2.id]);

      const state = useWorkspaceStore.getState();
      expect(state.tabs).toHaveLength(1);
      expect(state.tabs[0].id).toBe(tab1.id);
    });
  });

  describe("updateTabData", () => {
    it("updates query tab data by id", () => {
      const tab = createQueryTab("conn-1", "Q1", "SELECT 1");
      useWorkspaceStore.getState().openTab(tab);

      useWorkspaceStore.getState().updateTabData(tab.id, (d) => ({
        ...d,
        sql: "SELECT 2",
        status: "running",
      }));

      const state = useWorkspaceStore.getState();
      const updated = state.tabs[0];
      expect(updated.kind).toBe("query");
      if (updated.kind === "query") {
        expect(updated.data.sql).toBe("SELECT 2");
        expect(updated.data.status).toBe("running");
      }
    });

    it("does not affect other tabs", () => {
      const tab1 = createQueryTab("conn-1", "Q1", "SELECT 1");
      const tab2 = createQueryTab("conn-1", "Q2", "SELECT 2");
      const { openTab, updateTabData } = useWorkspaceStore.getState();

      openTab(tab1);
      openTab(tab2);
      updateTabData(tab1.id, (d) => ({ ...d, sql: "SELECT 99" }));

      const state = useWorkspaceStore.getState();
      const t2 = state.tabs.find((t) => t.id === tab2.id);
      expect(t2?.kind).toBe("query");
      if (t2?.kind === "query") {
        expect(t2.data.sql).toBe("SELECT 2");
      }
    });
  });

  describe("tab metadata", () => {
    it("setTabDirty updates dirty flag", () => {
      const tab = createQueryTab("conn-1", "Q1");
      useWorkspaceStore.getState().openTab(tab);
      useWorkspaceStore.getState().setTabDirty(tab.id, true);

      expect(useWorkspaceStore.getState().tabs[0].dirty).toBe(true);
    });

    it("toggleTabPinned flips pinned state", () => {
      const tab = createQueryTab("conn-1", "Q1");
      useWorkspaceStore.getState().openTab(tab);

      useWorkspaceStore.getState().toggleTabPinned(tab.id);
      expect(useWorkspaceStore.getState().tabs[0].pinned).toBe(true);

      useWorkspaceStore.getState().toggleTabPinned(tab.id);
      expect(useWorkspaceStore.getState().tabs[0].pinned).toBe(false);
    });

    it("promotePreview clears preview flag", () => {
      const tab = createQueryTab("conn-1", "Q1");
      tab.preview = true;
      useWorkspaceStore.getState().openTab(tab);
      useWorkspaceStore.getState().promotePreview(tab.id);

      expect(useWorkspaceStore.getState().tabs[0].preview).toBe(false);
    });

    it("setTabTitle updates title", () => {
      const tab = createQueryTab("conn-1", "Q1");
      useWorkspaceStore.getState().openTab(tab);
      useWorkspaceStore.getState().setTabTitle(tab.id, "Renamed");

      expect(useWorkspaceStore.getState().tabs[0].title).toBe("Renamed");
    });
  });

  describe("reorderTabs", () => {
    it("moves tab from one position to another", () => {
      const tab1 = createQueryTab("conn-1", "Q1");
      const tab2 = createQueryTab("conn-1", "Q2");
      const tab3 = createQueryTab("conn-1", "Q3");
      const { openTab, reorderTabs } = useWorkspaceStore.getState();

      openTab(tab1);
      openTab(tab2);
      openTab(tab3);
      reorderTabs(0, 2);

      const state = useWorkspaceStore.getState();
      expect(state.tabs[0].id).toBe(tab2.id);
      expect(state.tabs[1].id).toBe(tab3.id);
      expect(state.tabs[2].id).toBe(tab1.id);
    });

    it("refuses to move tabs into or from the pinned range", () => {
      const tab1 = createQueryTab("conn-1", "Q1");
      const tab2 = createQueryTab("conn-1", "Q2");
      const tab3 = createQueryTab("conn-1", "Q3");
      const { openTab, toggleTabPinned, reorderTabs } = useWorkspaceStore.getState();

      openTab(tab1);
      openTab(tab2);
      openTab(tab3);
      toggleTabPinned(tab1.id);

      reorderTabs(0, 2);

      const state = useWorkspaceStore.getState();
      expect(state.tabs[0].id).toBe(tab1.id);
      expect(state.tabs[1].id).toBe(tab2.id);
      expect(state.tabs[2].id).toBe(tab3.id);
    });
  });

  describe("restoreState", () => {
    it("restores tabs and activeTabId", () => {
      const tab = createQueryTab("conn-1", "Restored");
      useWorkspaceStore.getState().restoreState({
        tabs: [tab],
        activeTabId: tab.id,
        recentlyClosed: [],
      });

      const state = useWorkspaceStore.getState();
      expect(state.tabs).toHaveLength(1);
      expect(state.activeTabId).toBe(tab.id);
    });

    it("falls back to first tab if activeTabId not found", () => {
      const tab = createQueryTab("conn-1", "Q1");
      useWorkspaceStore.getState().restoreState({
        tabs: [tab],
        activeTabId: "nonexistent",
        recentlyClosed: [],
      });

      expect(useWorkspaceStore.getState().activeTabId).toBe(tab.id);
    });
  });

  describe("openDbObject", () => {
    it("replaces preview when opening a different preview", () => {
      const previewA = createDbObjectTab("conn-1", "public", "users", "table", "columns", true);
      const previewB = createDbObjectTab("conn-1", "public", "orders", "table", "columns", true);
      const { openDbObject } = useWorkspaceStore.getState();

      openDbObject(previewA);
      openDbObject(previewB);

      const state = useWorkspaceStore.getState();
      expect(state.tabs).toHaveLength(1);
      expect(state.tabs[0].title).toBe("orders");
      expect(state.tabs[0].preview).toBe(true);
      expect(state.tabs[0].id).toBe(previewA.id);
    });

    it("promotes preview and changes section on Open Data", () => {
      const preview = createDbObjectTab("conn-1", "public", "users", "table", "columns", true);
      const { openDbObject, promotePreview, setDbObjectSection, activateTab } = useWorkspaceStore.getState();

      openDbObject(preview);

      const existing = useWorkspaceStore.getState().tabs[0];
      expect(existing.preview).toBe(true);

      promotePreview(existing.id);
      setDbObjectSection(existing.id, "data");
      activateTab(existing.id);

      const state = useWorkspaceStore.getState();
      expect(state.tabs).toHaveLength(1);
      expect(state.tabs[0].preview).toBe(false);
      expect(state.tabs[0].id).toBe(preview.id);
      if (state.tabs[0].kind === "db-object") {
        expect(state.tabs[0].data.activeSection).toBe("data");
      }
    });

    it("ordinary click on existing tab preserves section", () => {
      const tab = createDbObjectTab("conn-1", "public", "users", "table", "columns", false);
      const { openDbObject, setDbObjectSection } = useWorkspaceStore.getState();

      openDbObject(tab);
      setDbObjectSection(tab.id, "data");

      const clickTab = createDbObjectTab("conn-1", "public", "users", "table", "columns", false);
      openDbObject(clickTab);

      const state = useWorkspaceStore.getState();
      expect(state.tabs).toHaveLength(1);
      expect(state.activeTabId).toBe(tab.id);
      if (state.tabs[0].kind === "db-object") {
        expect(state.tabs[0].data.activeSection).toBe("data");
      }
    });

    it("explicit setDbObjectSection changes section on persistent tab", () => {
      const tab = createDbObjectTab("conn-1", "public", "users", "table", "columns", false);
      const { openDbObject, setDbObjectSection } = useWorkspaceStore.getState();

      openDbObject(tab);
      setDbObjectSection(tab.id, "data");
      setDbObjectSection(tab.id, "ddl");

      const state = useWorkspaceStore.getState();
      if (state.tabs[0].kind === "db-object") {
        expect(state.tabs[0].data.activeSection).toBe("ddl");
      }
    });

    it("preserves objectType for views on Open Data", () => {
      const viewPreview = createDbObjectTab("conn-1", "public", "user_view", "view", "columns", true);
      const { openDbObject, promotePreview, setDbObjectSection } = useWorkspaceStore.getState();

      openDbObject(viewPreview);

      const existing = useWorkspaceStore.getState().tabs[0];
      promotePreview(existing.id);
      setDbObjectSection(existing.id, "data");

      const state = useWorkspaceStore.getState();
      expect(state.tabs).toHaveLength(1);
      if (state.tabs[0].kind === "db-object") {
        expect(state.tabs[0].data.objectType).toBe("view");
        expect(state.tabs[0].data.activeSection).toBe("data");
      }
    });

    it("does not create duplicate preview tabs across multiple clicks", () => {
      const { openDbObject } = useWorkspaceStore.getState();

      for (const name of ["users", "orders", "customers", "products"]) {
        const preview = createDbObjectTab("conn-1", "public", name, "table", "columns", true);
        openDbObject(preview);
      }

      const state = useWorkspaceStore.getState();
      expect(state.tabs).toHaveLength(1);
      expect(state.tabs[0].title).toBe("products");
    });

    it("resets grid state when preview slot is replaced with different resource", () => {
      const { openDbObject } = useWorkspaceStore.getState();

      const previewA = createDbObjectTab("conn-1", "public", "users", "table", "columns", true);
      openDbObject(previewA);

      const reusedId = useWorkspaceStore.getState().tabs[0].id;
      useTabGridStateStore.getState().setState(reusedId, { page: 8, filters: [{ column: "id", operator: "eq", value: "1", enabled: true }] });
      expect(useTabGridStateStore.getState().getState(reusedId).page).toBe(8);

      const previewB = createDbObjectTab("conn-1", "public", "orders", "table", "columns", true);
      openDbObject(previewB);

      expect(useWorkspaceStore.getState().tabs).toHaveLength(1);
      expect(useWorkspaceStore.getState().tabs[0].id).toBe(reusedId);
      const gridState = useTabGridStateStore.getState().getState(reusedId);
      expect(gridState.page).toBe(1);
      expect(gridState.filters).toHaveLength(0);
    });
  });
});
