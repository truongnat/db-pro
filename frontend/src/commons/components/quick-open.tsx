import { Command } from "cmdk";
import { useEffect, useMemo } from "react";
import { Columns3, Database, FileText, Folder, Table2 } from "lucide-react";

import {
  Dialog,
  DialogContent,
} from "@/components/ui/dialog";
import { useTranslation } from "@/commons/locales/useTranslation";
import { useCommandStore } from "@/commons/stores/command.store";
import { useConnectionStore } from "@/commons/stores/connection.store";
import { useExplorerStore } from "@/commons/stores/explorer.store";
import { useQuickOpenStore } from "@/commons/stores/quick-open.store";
import { useRecentStore } from "@/commons/stores/recent.store";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { useShellStore } from "@/commons/stores/shell.store";
import { useSidebarTabOps } from "@/commons/hooks/use-sidebar-tab-ops";
import { buildQuickOpenIndex } from "@/commons/services/quick-open-index";
import { rankQuickOpenItems } from "@/commons/services/quick-open-rank";
import type { QuickOpenItem } from "@/commons/types/quick-open.types";
import { useConnectionList, useConnect } from "@/modules/connection/queries/connection.queries";
import { useConnectionModuleStore } from "@/modules/connection/state/connection.store";
import { useSchemaCatalogStore } from "@/modules/query/stores/schema-catalog.store";

function groupItems(items: QuickOpenItem[]) {
  const order = ["tab", "table", "view", "schema", "connection"] as const;
  const groups = new Map<string, QuickOpenItem[]>();
  for (const item of items) {
    const key = item.kind === "db-object" ? item.objectType : item.kind;
    if (!groups.has(key)) groups.set(key, []);
    groups.get(key)!.push(item);
  }
  return order
    .filter((k) => groups.has(k) && groups.get(k)!.length > 0)
    .map((k) => ({ key: k, items: groups.get(k)! }));
}

export function QuickOpen() {
  const { t } = useTranslation();
  const isOpen = useQuickOpenStore((s) => s.isOpen);
  const query = useQuickOpenStore((s) => s.query);
  const setQuery = useQuickOpenStore((s) => s.setQuery);
  const close = useQuickOpenStore((s) => s.close);

  const connections = useConnectionList();
  const catalogs = useSchemaCatalogStore((s) => s.catalogs);
  const tabs = useWorkspaceStore((s) => s.tabs);
  const activeTabId = useWorkspaceStore((s) => s.activeTabId);
  const explorerConnectionId = useConnectionStore((s) => s.explorerConnectionId);
  const setExplorerConnection = useConnectionStore((s) => s.setExplorerConnection);
  const statuses = useConnectionModuleStore((s) => s.statuses);
  const expandNode = useExplorerStore((s) => s.expandNode);
  const recentResources = useRecentStore((s) => s.recentResources);
  const { openSchemaPreview } = useSidebarTabOps();
  const connectMutation = useConnect();

  useEffect(() => {
    if (!isOpen) return;

    // Preload catalogs for active/explorer connections once on open (not per keystroke)
    const ids = new Set<string>();
    if (explorerConnectionId) ids.add(explorerConnectionId);
    for (const tab of tabs) {
      if (tab.connectionId) ids.add(tab.connectionId);
    }
    for (const id of ids) {
      useSchemaCatalogStore.getState().ensureLoaded(id).catch(() => {
        /* catalog is best-effort */
      });
    }
  }, [isOpen, explorerConnectionId, tabs]);

  const index = useMemo(
    () => buildQuickOpenIndex({ connections: connections.data ?? [], catalogs, tabs }),
    [connections.data, catalogs, tabs],
  );

  const openResourceKeys = useMemo(
    () => new Set(tabs.map((t) => t.resourceKey)),
    [tabs],
  );
  const recentResourceKeys = useMemo(
    () => new Set(recentResources.map((r) => r.resourceKey)),
    [recentResources],
  );
  const activeTabConnectionId = useMemo(
    () => tabs.find((t) => t.id === activeTabId)?.connectionId ?? null,
    [tabs, activeTabId],
  );

  const ranked = useMemo(
    () =>
      rankQuickOpenItems(index, {
        query,
        activeTabId,
        activeConnectionId: activeTabConnectionId,
        explorerConnectionId,
        openResourceKeys,
        recentResourceKeys,
      }),
    [index, query, activeTabId, activeTabConnectionId, explorerConnectionId, openResourceKeys, recentResourceKeys],
  );

  const groups = useMemo(() => groupItems(ranked.map((r) => r.item)), [ranked]);

  useEffect(() => {
    if (!isOpen) return;
    useCommandStore.getState().close();
  }, [isOpen]);

  const handleOpenItem = (item: QuickOpenItem) => {
    switch (item.kind) {
      case "tab":
        useWorkspaceStore.getState().activateTab(item.tabId);
        break;
      case "db-object":
        openSchemaPreview(item.connectionId, item.schema, item.objectName, item.objectType);
        useRecentStore.getState().addRecentResource({
          resourceKey: item.resourceKey,
          kind: "db-object",
          connectionId: item.connectionId,
          schema: item.schema,
          objectName: item.objectName,
        });
        break;
      case "schema":
        setExplorerConnection(item.connectionId);
        useShellStore.getState().setSidebarView("explorer");
        expandNode(`conn:${item.connectionId}`);
        expandNode(`schema:${item.connectionId}:${item.schema}`);
        break;
      case "connection": {
        const status = statuses[item.connectionId] ?? "disconnected";
        if (status === "disconnected" || status === "error") {
          connectMutation.mutate(item.connectionId);
        } else {
          setExplorerConnection(item.connectionId);
        }
        useShellStore.getState().setSidebarView("explorer");
        expandNode(`conn:${item.connectionId}`);
        break;
      }
    }
    close();
  };

  const groupLabel = (key: string) => {
    switch (key) {
      case "tab":
        return t("quickOpen.groups.open");
      case "table":
        return t("quickOpen.groups.tables");
      case "view":
        return t("quickOpen.groups.views");
      case "schema":
        return t("quickOpen.groups.schemas");
      case "connection":
        return t("quickOpen.groups.connections");
      default:
        return "";
    }
  };

  const itemMeta = (item: QuickOpenItem): string => {
    switch (item.kind) {
      case "tab":
        return item.connectionName;
      case "db-object":
        return `${item.schema} · ${item.connectionName}`;
      case "schema":
        return item.connectionName;
      case "connection":
        return item.connectionName;
    }
  };

  const itemIcon = (item: QuickOpenItem) => {
    switch (item.kind) {
      case "tab":
        return <FileText className="h-4 w-4 shrink-0 text-[var(--app-text-muted)]" />;
      case "db-object":
        return item.objectType === "view" ? (
          <Columns3 className="h-4 w-4 shrink-0 text-primary" />
        ) : (
          <Table2 className="h-4 w-4 shrink-0 text-primary" />
        );
      case "schema":
        return <Folder className="h-4 w-4 shrink-0 text-primary" />;
      case "connection":
        return <Database className="h-4 w-4 shrink-0 text-[var(--app-text-muted)]" />;
    }
  };

  const itemTitle = (item: QuickOpenItem): string => {
    switch (item.kind) {
      case "tab":
        return item.title;
      case "db-object":
        return item.objectName;
      case "schema":
        return item.schema;
      case "connection":
        return item.connectionName;
    }
  };

  if (!isOpen) return null;

  return (
    <Dialog open={isOpen} onOpenChange={(open) => !open && close()}>
      <DialogContent className="overflow-hidden p-0 shadow-lg">
        <Command
          shouldFilter={false}
          className="[&_[cmdk-group-heading]]:px-2 [&_[cmdk-group-heading]]:py-1.5 [&_[cmdk-group-heading]]:text-xs [&_[cmdk-group-heading]]:font-medium [&_[cmdk-group-heading]]:text-[var(--app-text-muted)] [&_[cmdk-group]]:px-2 [&_[cmdk-group]]:py-1 [&_[cmdk-input-wrapper]]:px-3 [&_[cmdk-input-wrapper]]:py-2 [&_[cmdk-item]]:px-3 [&_[cmdk-item]]:py-2"
          label="Quick open"
        >
          <Command.Input
            placeholder={t("quickOpen.placeholder")}
            value={query}
            onValueChange={setQuery}
            className="flex h-10 w-full border-b border-[var(--app-border-subtle)] bg-transparent text-sm outline-none placeholder:text-[var(--app-text-dim)]"
            autoFocus
          />
          <Command.List className="max-h-[400px] overflow-y-auto overflow-x-hidden py-2">
            {groups.length === 0 && query.trim() && (
              <Command.Empty className="px-3 py-6 text-center text-sm text-[var(--app-text-muted)]">
                {t("quickOpen.noResults")}
              </Command.Empty>
            )}
            {groups.map((group) => (
              <Command.Group key={group.key} heading={groupLabel(group.key)}>
                {group.items.map((item) => (
                  <Command.Item
                    key={item.resourceKey}
                    value={item.resourceKey}
                    onSelect={() => handleOpenItem(item)}
                    className="relative flex cursor-default select-none items-center gap-2 rounded-sm text-sm outline-none aria-disabled:pointer-events-none aria-disabled:opacity-50 data-[selected=true]:bg-accent data-[selected=true]:text-accent-foreground"
                  >
                    {itemIcon(item)}
                    <span className="flex-1 truncate">{itemTitle(item)}</span>
                    <span className="truncate text-[11px] text-[var(--app-text-muted)]">
                      {itemMeta(item)}
                    </span>
                  </Command.Item>
                ))}
              </Command.Group>
            ))}
          </Command.List>
        </Command>
      </DialogContent>
    </Dialog>
  );
}
