import { Command } from "cmdk";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  Columns3,
  Database,
  FileText,
  Folder,
  FunctionSquare,
  Hash,
  Key,
  Table2,
  X,
} from "lucide-react";

import { Dialog, DialogContent } from "@/components/ui/dialog";
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
import { rankQuickOpenItems, type RankedQuickOpenItem } from "@/commons/services/quick-open-rank";
import type { QuickOpenItem } from "@/commons/types/quick-open.types";
import { useConnectionList, useConnect } from "@/modules/connection/queries/connection.queries";
import { useConnectionModuleStore } from "@/modules/connection/state/connection.store";
import { useSchemaCatalogStore } from "@/modules/query/stores/schema-catalog.store";

// ─── Grouping ────────────────────────────────────────────────────────────────

interface ItemGroup {
  key: string;
  items: RankedQuickOpenItem[];
}

function groupRankedItems(items: RankedQuickOpenItem[], recentKeys: Set<string>): ItemGroup[] {
  const order = [
    "recent",
    "tab",
    "table",
    "view",
    "function",
    "sequence",
    "type",
    "schema",
    "connection",
  ] as const;
  const groups = new Map<string, RankedQuickOpenItem[]>();
  for (const ranked of items) {
    const item = ranked.item;
    // Items in the recent set get grouped under "recent"
    const key = recentKeys.has(item.resourceKey)
      ? "recent"
      : item.kind === "db-object"
        ? item.objectType
        : item.kind;
    if (!groups.has(key)) groups.set(key, []);
    groups.get(key)!.push(ranked);
  }
  return order
    .filter((k) => groups.has(k) && groups.get(k)!.length > 0)
    .map((k) => ({ key: k, items: groups.get(k)! }));
}

// ─── Highlight ───────────────────────────────────────────────────────────────

function HighlightedText({ text, indices }: { text: string; indices: number[] }) {
  if (indices.length === 0) return <>{text}</>;
  const indexSet = new Set(indices);
  const parts: { text: string; highlighted: boolean }[] = [];
  let current = "";
  let currentHighlighted = false;

  for (let i = 0; i < text.length; i++) {
    const isHighlighted = indexSet.has(i);
    if (current === "") {
      currentHighlighted = isHighlighted;
      current = text[i];
    } else if (isHighlighted === currentHighlighted) {
      current += text[i];
    } else {
      parts.push({ text: current, highlighted: currentHighlighted });
      current = text[i];
      currentHighlighted = isHighlighted;
    }
  }
  if (current) parts.push({ text: current, highlighted: currentHighlighted });

  return (
    <>
      {parts.map((part, i) =>
        part.highlighted ? (
          <mark key={i} className="bg-transparent font-semibold text-primary">
            {part.text}
          </mark>
        ) : (
          <span key={i}>{part.text}</span>
        ),
      )}
    </>
  );
}

// ─── Prefix detection ────────────────────────────────────────────────────────

type PrefixFilter = "table" | "view" | "schema" | null;

function parsePrefix(query: string): { prefix: PrefixFilter; cleanQuery: string } {
  if (query.startsWith("@")) return { prefix: "table", cleanQuery: query.slice(1) };
  if (query.startsWith(":")) return { prefix: "view", cleanQuery: query.slice(1) };
  if (query.startsWith("#")) return { prefix: "schema", cleanQuery: query.slice(1) };
  return { prefix: null, cleanQuery: query };
}

function matchesPrefixFilter(item: QuickOpenItem, filter: PrefixFilter): boolean {
  if (!filter) return true;
  if (filter === "table" || filter === "view") {
    return item.kind === "db-object" && item.objectType === filter;
  }
  if (filter === "schema") {
    return item.kind === "schema";
  }
  return true;
}

// ─── Component ───────────────────────────────────────────────────────────────

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
  const removeRecentResource = useRecentStore((s) => s.removeRecentResource);
  const { openSchemaPreview } = useSidebarTabOps();
  const connectMutation = useConnect();

  // Track selected value for preview-on-navigate
  const [selectedValue, setSelectedValue] = useState<string>("");
  const lastPreviewedRef = useRef<string>("");
  const isInitialMountRef = useRef(true);

  // Parse prefix filter from query
  const { prefix, cleanQuery } = useMemo(() => parsePrefix(query), [query]);

  useEffect(() => {
    if (!isOpen) return;

    // Preload catalogs for active/explorer connections once on open
    const ids = new Set<string>();
    if (explorerConnectionId) ids.add(explorerConnectionId);
    for (const tab of tabs) {
      if (tab.connectionId) ids.add(tab.connectionId);
    }
    for (const id of ids) {
      useSchemaCatalogStore
        .getState()
        .ensureLoaded(id)
        .catch(() => {
          /* catalog is best-effort */
        });
    }
  }, [isOpen, explorerConnectionId, tabs]);

  const index = useMemo(
    () => buildQuickOpenIndex({ connections: connections.data ?? [], catalogs, tabs }),
    [connections.data, catalogs, tabs],
  );

  const openResourceKeys = useMemo(() => new Set(tabs.map((t) => t.resourceKey)), [tabs]);
  const recentResourceKeys = useMemo(
    () => new Set(recentResources.map((r) => r.resourceKey)),
    [recentResources],
  );
  const activeTabConnectionId = useMemo(
    () => tabs.find((t) => t.id === activeTabId)?.connectionId ?? null,
    [tabs, activeTabId],
  );

  const ranked = useMemo(() => {
    // Rank all items using the clean query (without prefix)
    let results = rankQuickOpenItems(index, {
      query: cleanQuery,
      activeTabId,
      activeConnectionId: activeTabConnectionId,
      explorerConnectionId,
      openResourceKeys,
      recentResourceKeys,
    });

    // Apply prefix filter
    if (prefix) {
      results = results.filter((r) => matchesPrefixFilter(r.item, prefix));
    }

    return results;
  }, [
    index,
    cleanQuery,
    activeTabId,
    activeTabConnectionId,
    explorerConnectionId,
    openResourceKeys,
    recentResourceKeys,
    prefix,
  ]);

  // Build recent items for empty-query recent group
  const recentRanked = useMemo<RankedQuickOpenItem[]>(() => {
    if (cleanQuery.trim() || prefix) return [];
    const result: RankedQuickOpenItem[] = [];
    const sources = recentResources.filter((r) => r.kind === "db-object").slice(0, 5);
    for (const r of sources) {
      const found = index.find((item) => item.resourceKey === r.resourceKey);
      if (found) {
        result.push({ item: found, score: 10000, matchIndices: [], titleMatchIndices: [] });
        continue;
      }
      if (!r.objectType || !r.objectName) continue;
      const connName =
        connections.data?.find((c) => c.id === r.connectionId)?.name ?? r.connectionId;
      result.push({
        item: {
          kind: "db-object" as const,
          connectionId: r.connectionId,
          connectionName: connName,
          schema: r.schema ?? "",
          objectName: r.objectName,
          objectType: r.objectType,
          resourceKey: r.resourceKey,
          searchText: `${r.objectName} ${r.schema ?? ""} ${connName}`,
        },
        score: 10000,
        matchIndices: [],
        titleMatchIndices: [],
      });
    }
    return result;
  }, [cleanQuery, prefix, recentResources, index, connections.data]);

  // Combine: recent group + ranked results
  const allRanked = useMemo(() => {
    if (recentRanked.length > 0) {
      // Filter out items from ranked that are already in recent group
      const recentKeys = new Set(recentRanked.map((r) => r.item.resourceKey));
      const filtered = ranked.filter((r) => !recentKeys.has(r.item.resourceKey));
      return [...recentRanked, ...filtered];
    }
    return ranked;
  }, [ranked, recentRanked]);

  const recentKeysSet = useMemo(
    () => new Set(recentRanked.map((r) => r.item.resourceKey)),
    [recentRanked],
  );
  const groups = useMemo(
    () => groupRankedItems(allRanked, recentKeysSet),
    [allRanked, recentKeysSet],
  );

  // Preview on navigate: when selected value changes, open as preview tab
  useEffect(() => {
    if (!isOpen || !selectedValue) return;
    if (isInitialMountRef.current) {
      isInitialMountRef.current = false;
      lastPreviewedRef.current = selectedValue;
      return;
    }
    if (selectedValue === lastPreviewedRef.current) return;
    const found = allRanked.find((r) => r.item.resourceKey === selectedValue);
    if (!found) return;
    const item = found.item;
    if (item.kind === "db-object") {
      lastPreviewedRef.current = selectedValue;
      openSchemaPreview(item.connectionId, item.schema, item.objectName, item.objectType);
    }
  }, [selectedValue, isOpen, allRanked, openSchemaPreview]);

  useEffect(() => {
    if (isOpen) {
      isInitialMountRef.current = true;
      useCommandStore.getState().close();
    } else {
      setSelectedValue("");
      lastPreviewedRef.current = "";
      isInitialMountRef.current = true;
    }
  }, [isOpen]);

  const handleOpenItem = useCallback(
    (item: QuickOpenItem) => {
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
            objectType: item.objectType,
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
    },
    [close, openSchemaPreview, setExplorerConnection, expandNode, statuses, connectMutation],
  );

  const handleRemoveRecentByKey = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key !== "Backspace" && e.key !== "Delete") return;
      if (!selectedValue) return;
      const isRecent = recentKeysSet.has(selectedValue);
      if (!isRecent) return;
      e.preventDefault();
      removeRecentResource(selectedValue);
    },
    [selectedValue, recentKeysSet, removeRecentResource],
  );

  const handleRemoveRecentClick = useCallback(
    (e: React.MouseEvent, resourceKey: string) => {
      e.stopPropagation();
      removeRecentResource(resourceKey);
    },
    [removeRecentResource],
  );

  const groupLabel = (key: string) => {
    switch (key) {
      case "recent":
        return t("quickOpen.groups.recent");
      case "tab":
        return t("quickOpen.groups.open");
      case "table":
        return t("quickOpen.groups.tables");
      case "view":
        return t("quickOpen.groups.views");
      case "function":
        return t("quickOpen.groups.functions");
      case "sequence":
        return t("quickOpen.groups.sequences");
      case "type":
        return t("quickOpen.groups.types");
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
        return `${item.schema}.${item.objectName} @ ${item.connectionName}`;
      case "schema":
        return item.connectionName;
      case "connection":
        return `${item.connectionName} · ${item.connectionId.slice(0, 8)}`;
    }
  };

  const itemIcon = (item: QuickOpenItem) => {
    const iconCls = "h-4 w-4 shrink-0";
    switch (item.kind) {
      case "tab":
        return <FileText className={`${iconCls} text-[var(--app-text-muted)]`} />;
      case "db-object":
        switch (item.objectType) {
          case "view":
            return <Columns3 className={`${iconCls} text-primary`} />;
          case "function":
            return <FunctionSquare className={`${iconCls} text-[var(--app-text-muted)]`} />;
          case "sequence":
            return <Hash className={`${iconCls} text-[var(--app-text-muted)]`} />;
          case "type":
            return <Key className={`${iconCls} text-[var(--app-text-muted)]`} />;
          default:
            return <Table2 className={`${iconCls} text-primary`} />;
        }
      case "schema":
        return <Folder className={`${iconCls} text-primary`} />;
      case "connection":
        return <Database className={`${iconCls} text-[var(--app-text-muted)]`} />;
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

  const showHint = !query.trim() && !prefix;
  const activePrefix = prefix;

  return (
    <Dialog open={isOpen} onOpenChange={(open) => !open && close()}>
      <DialogContent className="!w-[680px] !max-w-[680px] -translate-x-1/2 overflow-hidden p-0 shadow-lg">
        <Command
          shouldFilter={false}
          value={selectedValue}
          onValueChange={setSelectedValue}
          onKeyDown={handleRemoveRecentByKey}
          className="[&_[cmdk-group-heading]]:px-2 [&_[cmdk-group-heading]]:py-1.5 [&_[cmdk-group-heading]]:text-xs [&_[cmdk-group-heading]]:font-medium [&_[cmdk-group-heading]]:text-[var(--app-text-muted)] [&_[cmdk-group]]:px-2 [&_[cmdk-group]]:py-1 [&_[cmdk-input-wrapper]]:px-3 [&_[cmdk-input-wrapper]]:py-2 [&_[cmdk-item]]:px-3 [&_[cmdk-item]]:py-2"
          label="Quick open"
        >
          <Command.Input
            placeholder={t("quickOpen.placeholder")}
            value={query}
            onValueChange={(v) => {
              setQuery(v);
              setSelectedValue("");
              lastPreviewedRef.current = "";
            }}
            className="flex h-10 w-full border-b border-[var(--app-border-subtle)] bg-transparent px-3 text-sm outline-none placeholder:text-[var(--app-text-dim)]"
            autoFocus
          />
          <Command.List className="max-h-[500px] overflow-y-auto overflow-x-hidden py-2">
            {showHint && (
              <div className="px-3 pb-1.5 text-[11px] text-[var(--app-text-dim)]">
                {t("quickOpen.hints.prefix")}
              </div>
            )}
            {activePrefix && (
              <div className="flex items-center gap-1 px-3 pb-1 text-[11px] text-primary">
                <span className="font-mono font-semibold">
                  {activePrefix === "table" ? "@" : activePrefix === "view" ? ":" : "#"}
                </span>
                <span>
                  {activePrefix === "table"
                    ? t("quickOpen.groups.tables")
                    : activePrefix === "view"
                      ? t("quickOpen.groups.views")
                      : t("quickOpen.groups.schemas")}
                </span>
              </div>
            )}
            {groups.length === 0 && (cleanQuery.trim() || prefix) && (
              <Command.Empty className="px-3 py-6 text-center text-sm text-[var(--app-text-muted)]">
                {t("quickOpen.noResults")}
              </Command.Empty>
            )}
            {groups.map((group) => (
              <Command.Group key={group.key} heading={groupLabel(group.key)}>
                {group.items.map((ranked) => {
                  const item = ranked.item;
                  const title = itemTitle(item);
                  const meta = itemMeta(item);
                  const isRecent = group.key === "recent";

                  return (
                    <Command.Item
                      key={item.resourceKey}
                      value={item.resourceKey}
                      onSelect={() => handleOpenItem(item)}
                      className="group relative flex cursor-default select-none items-center gap-2 rounded-sm text-sm outline-none aria-disabled:pointer-events-none aria-disabled:opacity-50 data-[selected=true]:bg-accent data-[selected=true]:text-accent-foreground"
                    >
                      {itemIcon(item)}
                      <span className="min-w-0 flex-1 break-words text-sm leading-snug">
                        <HighlightedText text={title} indices={ranked.titleMatchIndices} />
                      </span>
                      <span className="flex shrink-0 items-center gap-1">
                        <span className="max-w-[220px] truncate text-[11px] text-[var(--app-text-muted)]">
                          {meta}
                        </span>
                        {isRecent && (
                          <button
                            type="button"
                            className="flex h-4 w-4 items-center justify-center rounded text-[var(--app-text-dim)] opacity-0 transition-opacity group-hover:opacity-100 data-[selected=true]:opacity-60 hover:text-foreground"
                            aria-label={t("quickOpen.removeRecent")}
                            onClick={(e) => handleRemoveRecentClick(e, item.resourceKey)}
                          >
                            <X className="h-3 w-3 shrink-0" aria-hidden />
                          </button>
                        )}
                      </span>
                    </Command.Item>
                  );
                })}
              </Command.Group>
            ))}
          </Command.List>
        </Command>
      </DialogContent>
    </Dialog>
  );
}
