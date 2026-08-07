import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { createQueryTab, createTableDataTab } from "@/commons/factories/tab-factories";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";

function resetStore() {
  useWorkspaceStore.setState({
    tabs: [],
    activeTabId: null,
    recentlyClosed: [],
  });
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
      const tab1 = createTableDataTab("conn-1", "public", "users");
      const tab2 = createTableDataTab("conn-1", "public", "users");

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
  });

  describe("reopenLastClosed", () => {
    it("reopens the last closed tab with a new id", () => {
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
      expect(reopened!.id).not.toBe(tab1.id);
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
});
