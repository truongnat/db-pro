import type { AnyRouter } from "@tanstack/react-router";

import { dispatchQueryAction } from "@/commons/commands/query-dispatch";
import { createQueryTab } from "@/commons/factories/tab-factories";
import { requestCloseTab } from "@/commons/services/request-close-tab";
import { useConnectionStore } from "@/commons/stores/connection.store";
import { useCommandStore } from "@/commons/stores/command.store";
import { useShellStore } from "@/commons/stores/shell.store";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import type { QueryTabData, WorkspaceTab } from "@/commons/types/workspace.types";

let registered = false;

export function registerAllCommands(router: AnyRouter): void {
  if (registered) return;
  registered = true;

  const hasConnection = () => !!useConnectionStore.getState().activeConnectionId;
  const hasSql = () => {
    const tab = getActiveQueryTab();
    return !!tab && tab.data.sql.trim().length > 0;
  };
  const isRunning = () => {
    const tab = getActiveQueryTab();
    return !!tab && tab.data.status === "running";
  };
  const hasResults = () => {
    const tab = getActiveQueryTab();
    return !!tab && tab.data.result !== null;
  };

  function getActiveQueryTab(): (Omit<WorkspaceTab, "data"> & { data: QueryTabData }) | undefined {
    const { tabs, activeTabId } = useWorkspaceStore.getState();
    const tab = tabs.find((t) => t.id === activeTabId && t.kind === "query");
    return tab as (Omit<WorkspaceTab, "data"> & { data: QueryTabData }) | undefined;
  }

  function navigate(path: string) {
    router.navigate({ to: path });
  }

  useCommandStore.getState().registerMany([
    {
      id: "query.execute",
      labelKey: "commands.query.execute",
      keybinding: { ctrlKey: true, key: "Enter" },
      groupKey: "commands.groups.query",
      when: () => hasConnection() && hasSql(),
      execute: () => dispatchQueryAction("execute"),
    },
    {
      id: "query.explain",
      labelKey: "commands.query.explain",
      groupKey: "commands.groups.query",
      when: () => hasConnection() && hasSql(),
      execute: () => dispatchQueryAction("explain"),
    },
    {
      id: "query.format",
      labelKey: "commands.query.format",
      groupKey: "commands.groups.query",
      when: () => hasSql(),
      execute: () => dispatchQueryAction("format"),
    },
    {
      id: "query.clear",
      labelKey: "commands.query.clear",
      groupKey: "commands.groups.query",
      when: () => hasSql(),
      execute: () => dispatchQueryAction("clear"),
    },
    {
      id: "query.cancel",
      labelKey: "commands.query.cancel",
      groupKey: "commands.groups.query",
      when: () => hasConnection() && isRunning(),
      execute: () => dispatchQueryAction("cancel"),
    },
    {
      id: "query.exportResults",
      labelKey: "commands.query.exportResults",
      groupKey: "commands.groups.query",
      when: () => hasResults(),
      execute: () => dispatchQueryAction("export"),
    },
    {
      id: "query.saveQuery",
      labelKey: "commands.query.saveQuery",
      groupKey: "commands.groups.query",
      when: () => hasConnection() && hasSql(),
      execute: () => dispatchQueryAction("saveQuery"),
    },

    {
      id: "tabs.new",
      labelKey: "commands.tabs.new",
      groupKey: "commands.groups.tabs",
      when: () => hasConnection(),
      execute: () => {
        const connectionId = useConnectionStore.getState().activeConnectionId;
        if (!connectionId) return;
        useWorkspaceStore.getState().openTab(createQueryTab(connectionId));
      },
    },
    {
      id: "tabs.close",
      labelKey: "commands.tabs.close",
      groupKey: "commands.groups.tabs",
      when: () => !!useWorkspaceStore.getState().activeTabId,
      execute: () => {
        const { activeTabId } = useWorkspaceStore.getState();
        if (activeTabId) requestCloseTab(activeTabId);
      },
    },
    {
      id: "tabs.reopen",
      labelKey: "commands.tabs.reopen",
      keybinding: { ctrlKey: true, shiftKey: true, key: "t" },
      groupKey: "commands.groups.tabs",
      when: () => useWorkspaceStore.getState().recentlyClosed.length > 0,
      execute: () => useWorkspaceStore.getState().reopenLastClosed(),
    },

    {
      id: "nav.connections",
      labelKey: "commands.nav.connections",
      groupKey: "commands.groups.navigation",
      execute: () => navigate("/connections"),
    },
    {
      id: "nav.query",
      labelKey: "commands.nav.query",
      groupKey: "commands.groups.navigation",
      execute: () => navigate("/query"),
    },
    {
      id: "nav.data",
      labelKey: "commands.nav.data",
      groupKey: "commands.groups.navigation",
      execute: () => navigate("/data"),
    },
    {
      id: "nav.schema",
      labelKey: "commands.nav.schema",
      groupKey: "commands.groups.navigation",
      execute: () => navigate("/schema"),
    },
    {
      id: "nav.users",
      labelKey: "commands.nav.users",
      groupKey: "commands.groups.navigation",
      execute: () => navigate("/users"),
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
  ]);
}

export function resetCommandRegistration(): void {
  registered = false;
}
