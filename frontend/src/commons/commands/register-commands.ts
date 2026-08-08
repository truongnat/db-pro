import type { AnyRouter } from "@tanstack/react-router";

import { dispatchQueryAction } from "@/commons/commands/query-dispatch";
import { useCloseGuardStore } from "@/commons/stores/close-guard.store";
import { useConnectionStore } from "@/commons/stores/connection.store";
import { useCommandStore } from "@/commons/stores/command.store";
import { useQueryHistoryStore } from "@/commons/stores/query-history.store";
import { useShellStore } from "@/commons/stores/shell.store";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import {
  createQueryTabFromExplorerContext,
  getActiveQueryTab,
} from "@/modules/query/controllers/query-workspace.controller";
import { commandFromAction, executeAction } from "@/commons/actions";
import { useQueryEditorContextStore } from "@/commons/stores/query-editor-context.store";
import type { Keybinding } from "@/commons/types/command.types";

let registered = false;

export function registerAllCommands(_router: AnyRouter): void {
  if (registered) return;
  registered = true;

  const hasConnection = () => !!useConnectionStore.getState().explorerConnectionId;
  const hasSql = () => {
    const tab = getActiveQueryTab();
    return !!tab && tab.data.sql.trim().length > 0;
  };
  const hasResults = () => {
    const tab = getActiveQueryTab();
    return !!tab && tab.data.result !== null;
  };

  function requestCloseMany(ids: string[]) {
    const { tabs } = useWorkspaceStore.getState();
    const dirtyIds = ids.filter((id) => tabs.find((t) => t.id === id)?.dirty);
    if (dirtyIds.length === 0) {
      useWorkspaceStore.getState().closeTabs(ids);
    } else {
      useCloseGuardStore.getState().openDialog(ids, dirtyIds.length);
    }
  }

  /**
   * Helper: create a Command from an Action, optionally adding a keybinding.
   * This is the gradual migration path — commands that have Action Platform
   * equivalents delegate to the action bus; the rest stay as-is.
   */
  function actionCommand(
    actionId: string,
    keybinding?: Keybinding,
    options?: { inputProvider?: () => Record<string, unknown> | undefined },
  ) {
    const cmd = commandFromAction(actionId, options);
    if (keybinding) cmd.keybinding = keybinding;
    return cmd;
  }

  useCommandStore.getState().registerMany([
    // ── Query — delegated to Action Platform ──────────────────
    actionCommand("query.execute.current", { ctrlKey: true, key: "Enter" }, {
      inputProvider: () => {
        const tab = getActiveQueryTab();
        if (!tab) return undefined;
        // Read real cursor state from the editor context store.
        const editorCtx = useQueryEditorContextStore.getState().getEditorContext(tab.id);
        return {
          tabId: tab.id,
          cursorOffset: editorCtx.cursorOffset,
          selection: editorCtx.selection,
        };
      },
    }),
    actionCommand("query.explain"),
    actionCommand("query.format"),
    actionCommand("query.clear"),
    actionCommand("query.cancel"),
    // query.save requires a name — Command Palette cannot provide one.
    // Do not expose a broken command. The toolbar Save button handles this.
    // actionCommand("query.save"),  // disabled until dialog flow exists

    // ── Query — still using dispatch (no direct action yet) ──
    {
      id: "query.exportResults",
      labelKey: "commands.query.exportResults",
      groupKey: "commands.groups.query",
      when: () => hasResults(),
      execute: () => dispatchQueryAction("export"),
    },

    {
      id: "tabs.new",
      labelKey: "commands.tabs.new",
      keybinding: { ctrlKey: true, key: "t" },
      groupKey: "commands.groups.tabs",
      when: () => hasConnection(),
      execute: () => {
        const connectionId = useConnectionStore.getState().explorerConnectionId;
        if (!connectionId) return;
        const tab = createQueryTabFromExplorerContext(connectionId);
        if (tab) useWorkspaceStore.getState().openTab(tab);
      },
    },
    // ── Tabs — close/pin delegated to Action Platform ────────
    actionCommand("workspace.tab.close", { ctrlKey: true, key: "w" }),
    actionCommand("workspace.tab.pin", { altKey: true, shiftKey: true, key: "p" }),

    // ── Tabs — remaining still imperative ─────────────────────
    {
      id: "tabs.reopen",
      labelKey: "commands.tabs.reopen",
      keybinding: { ctrlKey: true, shiftKey: true, key: "t" },
      groupKey: "commands.groups.tabs",
      when: () => useWorkspaceStore.getState().recentlyClosed.length > 0,
      execute: () => useWorkspaceStore.getState().reopenLastClosed(),
    },
    {
      id: "tabs.closeOthers",
      labelKey: "commands.tabs.closeOthers",
      groupKey: "commands.groups.tabs",
      when: () => useWorkspaceStore.getState().tabs.length > 1,
      execute: () => {
        const { activeTabId, tabs } = useWorkspaceStore.getState();
        if (!activeTabId) return;
        const evictionIds = tabs.filter((t) => t.id !== activeTabId && !t.pinned).map((t) => t.id);
        if (evictionIds.length === 0) return;
        requestCloseMany(evictionIds);
      },
    },
    {
      id: "tabs.closeRight",
      labelKey: "commands.tabs.closeRight",
      groupKey: "commands.groups.tabs",
      when: () => {
        const { activeTabId, tabs } = useWorkspaceStore.getState();
        if (!activeTabId) return false;
        const idx = tabs.findIndex((t) => t.id === activeTabId);
        return idx >= 0 && idx < tabs.length - 1;
      },
      execute: () => {
        const { activeTabId, tabs } = useWorkspaceStore.getState();
        if (!activeTabId) return;
        const idx = tabs.findIndex((t) => t.id === activeTabId);
        const evictionIds = tabs.filter((t, i) => i > idx && !t.pinned).map((t) => t.id);
        if (evictionIds.length === 0) return;
        requestCloseMany(evictionIds);
      },
    },

    {
      id: "nav.connections",
      labelKey: "commands.nav.connections",
      groupKey: "commands.groups.navigation",
      execute: () => useShellStore.getState().setSidebarView("explorer"),
    },
    {
      id: "nav.query",
      labelKey: "commands.nav.query",
      groupKey: "commands.groups.navigation",
      when: () => hasConnection(),
      execute: () => {
        const connectionId = useConnectionStore.getState().explorerConnectionId;
        if (!connectionId) return;
        const { tabs, openTab, activateTab } = useWorkspaceStore.getState();
        const existing = tabs.find((t) => t.kind === "query" && t.connectionId === connectionId);
        if (!existing) {
          const tab = createQueryTabFromExplorerContext(connectionId);
          if (tab) openTab(tab);
        } else {
          activateTab(existing.id);
        }
      },
    },
    {
      id: "nav.data",
      labelKey: "commands.nav.data",
      groupKey: "commands.groups.navigation",
      execute: () => useShellStore.getState().setSidebarView("explorer"),
    },
    {
      id: "nav.schema",
      labelKey: "commands.nav.schema",
      groupKey: "commands.groups.navigation",
      execute: () => useShellStore.getState().setSidebarView("explorer"),
    },
    {
      id: "nav.users",
      labelKey: "commands.nav.users",
      groupKey: "commands.groups.navigation",
      execute: () => useShellStore.getState().setSidebarView("users"),
    },

    {
      id: "shell.toggleSidebar",
      labelKey: "commands.shell.toggleSidebar",
      groupKey: "commands.groups.shell",
      execute: () => useShellStore.getState().toggleSidebar(),
    },
    {
      id: "shell.settings",
      labelKey: "commands.shell.settings",
      groupKey: "commands.groups.shell",
      execute: () => {
        const { toggle } = useCommandStore.getState();
        toggle();
      },
    },

    {
      id: "file.importSql",
      labelKey: "commands.file.importSql",
      groupKey: "commands.groups.file",
      execute: () => dispatchQueryAction("importSql"),
    },
    {
      id: "file.exportSql",
      labelKey: "commands.file.exportSql",
      groupKey: "commands.groups.file",
      when: () => hasSql(),
      execute: () => dispatchQueryAction("exportSql"),
    },

    // ── Connection — delegated to Action Platform ─────────────
    actionCommand("connection.new"),

    // ── Productivity — panel actions delegated ────────────────
    {
      id: "productivity.clearHistory",
      labelKey: "commands.productivity.clearHistory",
      groupKey: "commands.groups.productivity",
      execute: () => useQueryHistoryStore.getState().clearHistory(),
    },
    {
      id: "productivity.showSnippets",
      labelKey: "commands.productivity.showSnippets",
      groupKey: "commands.groups.productivity",
      when: () => !!getActiveQueryTab(),
      execute: () => {
        executeAction("workspace.panel.set", { panel: "snippets" }, { source: "command-palette" });
      },
    },
    {
      id: "productivity.showHistory",
      labelKey: "commands.productivity.showHistory",
      groupKey: "commands.groups.productivity",
      when: () => !!getActiveQueryTab(),
      execute: () => {
        executeAction("workspace.panel.set", { panel: "history" }, { source: "command-palette" });
      },
    },
  ]);
}

export function resetCommandRegistration(): void {
  registered = false;
}
