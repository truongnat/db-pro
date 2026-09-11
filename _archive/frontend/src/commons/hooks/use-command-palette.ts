import { useEffect } from "react";

import { useCommandStore } from "@/commons/stores/command.store";
import type { Keybinding } from "@/commons/types/command.types";

function matchesKeybinding(event: KeyboardEvent, keybinding: Keybinding): boolean {
  if (event.key.toLowerCase() !== keybinding.key.toLowerCase()) {
    return false;
  }
  const wantCtrlOrCmd =
    (keybinding.primary ?? false) || (keybinding.ctrlKey ?? false) || (keybinding.metaKey ?? false);
  if (wantCtrlOrCmd && !event.ctrlKey && !event.metaKey) {
    return false;
  }
  if (!wantCtrlOrCmd && (event.ctrlKey || event.metaKey)) {
    return false;
  }
  if (event.shiftKey !== (keybinding.shiftKey ?? false)) {
    return false;
  }
  if (event.altKey !== (keybinding.altKey ?? false)) {
    return false;
  }
  return true;
}

export function useCommandPalette() {
  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      const state = useCommandStore.getState();

      // Cmd/Ctrl+Shift+P to toggle command palette (Cmd/Ctrl+P is Quick Open)
      if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key.toLowerCase() === "p") {
        e.preventDefault();
        state.toggle();
        return;
      }

      // Escape to close palette
      if (e.key === "Escape" && state.isOpen) {
        e.preventDefault();
        state.close();
        return;
      }

      // When palette is closed, dispatch registered keybindings
      if (!state.isOpen) {
        const commands = state.getAvailableCommands();
        for (const cmd of commands) {
          if (!cmd.keybinding) continue;
          if (matchesKeybinding(e, cmd.keybinding)) {
            e.preventDefault();
            state.executeCommand(cmd.id);
            return;
          }
        }
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);
}
