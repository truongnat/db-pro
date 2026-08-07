import { useEffect } from "react";

import { useCommandStore } from "@/commons/stores/command.store";
import type { Keybinding } from "@/commons/types/command.types";

function matchesKeybinding(event: KeyboardEvent, keybinding: Keybinding): boolean {
  if (event.key.toLowerCase() !== keybinding.key.toLowerCase()) {
    return false;
  }
  const wantCtrlOrCmd = (keybinding.ctrlKey ?? false) || (keybinding.metaKey ?? false);
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

      // Ctrl+K or Cmd+K to toggle palette
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
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
