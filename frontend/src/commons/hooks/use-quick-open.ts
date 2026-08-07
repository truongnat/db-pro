import { useEffect } from "react";

import { useQuickOpenStore } from "@/commons/stores/quick-open.store";

export function useQuickOpen() {
  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      const state = useQuickOpenStore.getState();

      // Cmd/Ctrl+P to open quick open (or keep focused if already open)
      if ((e.ctrlKey || e.metaKey) && !e.shiftKey && e.key.toLowerCase() === "p") {
        e.preventDefault();
        if (!state.isOpen) {
          state.open();
        }
        return;
      }

      // Escape to close quick open
      if (e.key === "Escape" && state.isOpen) {
        e.preventDefault();
        state.close();
        return;
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);
}
