import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { useShellStore } from "../shell.store";

function resetStore() {
  useShellStore.setState({ sidebarCollapsed: false, sidebarView: "explorer" });
}

describe("ShellStore", () => {
  beforeEach(resetStore);
  afterEach(resetStore);

  describe("initial state", () => {
    it("starts with sidebar expanded and explorer view", () => {
      const state = useShellStore.getState();
      expect(state.sidebarCollapsed).toBe(false);
      expect(state.sidebarView).toBe("explorer");
    });
  });

  describe("toggleSidebar", () => {
    it("toggles sidebar collapsed state", () => {
      useShellStore.getState().toggleSidebar();
      expect(useShellStore.getState().sidebarCollapsed).toBe(true);
      useShellStore.getState().toggleSidebar();
      expect(useShellStore.getState().sidebarCollapsed).toBe(false);
    });
  });

  describe("setSidebarCollapsed", () => {
    it("sets sidebar collapsed explicitly", () => {
      useShellStore.getState().setSidebarCollapsed(true);
      expect(useShellStore.getState().sidebarCollapsed).toBe(true);
      useShellStore.getState().setSidebarCollapsed(false);
      expect(useShellStore.getState().sidebarCollapsed).toBe(false);
    });
  });

  describe("setSidebarView", () => {
    it("changes sidebar view to search", () => {
      useShellStore.getState().setSidebarView("search");
      expect(useShellStore.getState().sidebarView).toBe("search");
    });

    it("changes sidebar view to query-saved", () => {
      useShellStore.getState().setSidebarView("query-saved");
      expect(useShellStore.getState().sidebarView).toBe("query-saved");
    });

    it("changes sidebar view to users", () => {
      useShellStore.getState().setSidebarView("users");
      expect(useShellStore.getState().sidebarView).toBe("users");
    });

    it("changes back to explorer", () => {
      useShellStore.getState().setSidebarView("search");
      useShellStore.getState().setSidebarView("explorer");
      expect(useShellStore.getState().sidebarView).toBe("explorer");
    });
  });
});
