import { useEffect } from "react";

import { useWorkspaceStore } from "@/commons/stores/workspace.store";

export function useTabKeyboard() {
  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      if (!e.ctrlKey && !e.metaKey) return;

      const { tabs, activeTabId, activateTab, reopenLastClosed } =
        useWorkspaceStore.getState();

      if (tabs.length === 0 && e.key !== "T") return;

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

      if (e.shiftKey && (e.key === "T" || e.key === "t")) {
        e.preventDefault();
        reopenLastClosed();
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);
}
