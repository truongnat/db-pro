import { useEffect } from "react";

import { useWorkspaceStore } from "@/commons/stores/workspace.store";

export function getTabNavigationOrder<T extends { pinned: boolean }>(tabs: T[]): T[] {
  return [...tabs.filter((tab) => tab.pinned), ...tabs.filter((tab) => !tab.pinned)];
}

export function useTabKeyboard() {
  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      if (!e.ctrlKey && !e.metaKey) return;

      const { tabs: rawTabs, activeTabId, activateTab } = useWorkspaceStore.getState();
      const tabs = getTabNavigationOrder(rawTabs);

      if (tabs.length === 0) return;

      if (e.key === "Tab" && !e.shiftKey) {
        e.preventDefault();
        const idx = tabs.findIndex((t) => t.id === activeTabId);
        const next = (idx + 1) % tabs.length;
        activateTab(tabs[next].id);
        return;
      }

      if (e.key === "Tab" && e.shiftKey) {
        e.preventDefault();
        const idx = tabs.findIndex((t) => t.id === activeTabId);
        const prev = (idx - 1 + tabs.length) % tabs.length;
        activateTab(tabs[prev].id);
        return;
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);
}
