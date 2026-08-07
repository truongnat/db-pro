import type { AnyRouter } from "@tanstack/react-router";

import { dispatchQueryAction } from "@/commons/commands/query-dispatch";
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
      label: "Execute Query",
      keybinding: { ctrlKey: true, key: "Enter" },
      group: "Query",
      when: () => hasConnection() && hasSql(),
      execute: () => dispatchQueryAction("execute"),
    },
    {
      id: "query.explain",
      label: "Explain Plan",
      group: "Query",
      when: () => hasConnection() && hasSql(),
      execute: () => dispatchQueryAction("explain"),
    },
    {
      id: "query.format",
      label: "Format SQL",
      group: "Query",
      when: () => hasSql(),
      execute: () => dispatchQueryAction("format"),
    },
    {
      id: "query.clear",
      label: "Clear Editor",
      group: "Query",
      when: () => hasSql(),
      execute: () => dispatchQueryAction("clear"),
    },
    {
      id: "query.cancel",
      label: "Cancel Query",
      group: "Query",
      when: () => hasConnection() && isRunning(),
      execute: () => dispatchQueryAction("cancel"),
    },
    {
      id: "query.exportResults",
      label: "Export Results",
      group: "Query",
      when: () => hasResults(),
      execute: () => dispatchQueryAction("export"),
    },
    {
      id: "query.saveQuery",
      label: "Save Query",
      group: "Query",
      when: () => hasConnection() && hasSql(),
      execute: () => dispatchQueryAction("saveQuery"),
    },

    {
      id: "tabs.new",
      label: "New Tab",
      group: "Tabs",
      execute: () => {
        const tab: WorkspaceTab = {
          id: crypto.randomUUID(),
          kind: "query",
          title: "New Query",
          connectionId: useConnectionStore.getState().activeConnectionId ?? "",
          resourceKey: `query:${crypto.randomUUID()}`,
          dirty: false,
          pinned: false,
          preview: false,
          order: Date.now(),
          data: {
            sql: "",
            status: "idle",
            error: null,
            result: null,
            explainPlan: null,
            sort: { column: null, direction: null },
            multiResults: null,
            multiResultIndex: 0,
          },
        };
        useWorkspaceStore.getState().openTab(tab);
      },
    },
    {
      id: "tabs.close",
      label: "Close Tab",
      group: "Tabs",
      when: () => !!useWorkspaceStore.getState().activeTabId,
      execute: () => {
        const { activeTabId } = useWorkspaceStore.getState();
        if (activeTabId) useWorkspaceStore.getState().closeTab(activeTabId);
      },
    },
    {
      id: "tabs.reopen",
      label: "Reopen Closed Tab",
      keybinding: { ctrlKey: true, shiftKey: true, key: "t" },
      group: "Tabs",
      when: () => useWorkspaceStore.getState().recentlyClosed.length > 0,
      execute: () => useWorkspaceStore.getState().reopenLastClosed(),
    },

    {
      id: "nav.connections",
      label: "Go to Connections",
      group: "Navigation",
      execute: () => navigate("/connections"),
    },
    {
      id: "nav.query",
      label: "Go to Query Editor",
      group: "Navigation",
      execute: () => navigate("/query"),
    },
    {
      id: "nav.data",
      label: "Go to Data",
      group: "Navigation",
      execute: () => navigate("/data"),
    },
    {
      id: "nav.schema",
      label: "Go to Schema",
      group: "Navigation",
      execute: () => navigate("/schema"),
    },
    {
      id: "nav.users",
      label: "Go to Users",
      group: "Navigation",
      execute: () => navigate("/users"),
    },

    {
      id: "shell.toggleSidebar",
      label: "Toggle Sidebar",
      group: "Shell",
      execute: () => useShellStore.getState().toggleSidebar(),
    },
    {
      id: "shell.settings",
      label: "Open Settings",
      group: "Shell",
      execute: () => {
        const { toggle } = useCommandStore.getState();
        toggle();
      },
    },

    {
      id: "file.importSql",
      label: "Import SQL File",
      group: "File",
      execute: () => dispatchQueryAction("importSql"),
    },
    {
      id: "file.exportSql",
      label: "Export SQL File",
      group: "File",
      when: () => hasSql(),
      execute: () => dispatchQueryAction("exportSql"),
    },
  ]);
}

export function resetCommandRegistration(): void {
  registered = false;
}
