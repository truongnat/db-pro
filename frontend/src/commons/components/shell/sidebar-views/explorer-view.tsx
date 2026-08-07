import {
  ChevronDown,
  ChevronRight,
  Columns3,
  Copy,
  Folder,
  FolderOpen,
  Plus,
  Search,
  Table2,
} from "lucide-react";

import { useTranslation } from "@/commons/locales/useTranslation";
import { useConnectionStore } from "@/commons/stores/connection.store";
import { useExplorerStore } from "@/commons/stores/explorer.store";
import { useRecentStore } from "@/commons/stores/recent.store";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { useSidebarTabOps } from "@/commons/hooks/use-sidebar-tab-ops";
import { StatusDot } from "@/commons/components/shell/status-dot";
import { Button } from "@/components/ui/button";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuSub,
  ContextMenuSubContent,
  ContextMenuSubTrigger,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";
import { useConnectionList, useConnect } from "@/modules/connection/queries/connection.queries";
import { useConnectionModuleStore } from "@/modules/connection/state/connection.store";
import { useIntrospect } from "@/modules/schema/queries/schema.queries";
import { createQueryTabForObject } from "@/modules/query/controllers/query-workspace.controller";
import { getSqlDialect } from "@/modules/query/sql/dialect";
import { generateCountSQL } from "@/modules/query/sql/generators";
import type { Connection, DriverType } from "@/modules/connection/types/connection.types";

function statusOf(statuses: Record<string, string>, id: string) {
  return statuses[id] ?? "disconnected";
}

function openNewQueryWithSql(connectionId: string, schema: string, objectName: string, sql: string) {
  const tab = createQueryTabForObject(connectionId, schema, { sql });
  useWorkspaceStore.getState().openTab(tab);
}

function copyToClipboard(text: string) {
  navigator.clipboard.writeText(text);
}

function getDriverForConnection(connectionId: string): DriverType {
  const connections = useConnectionStore.getState().connections as Connection[];
  return connections.find((c) => c.id === connectionId)?.driver ?? "postgres";
}

interface SchemaObjectGroupProps {
  groupKey: string;
  label: string;
  count: number;
  icon: React.ReactNode;
  expandedNodes: string[];
  onToggle: (key: string) => void;
  children: React.ReactNode;
}

function SchemaObjectGroup({ groupKey, label, count, icon, expandedNodes, onToggle, children }: SchemaObjectGroupProps) {
  const isOpen = expandedNodes.includes(groupKey);
  return (
    <div>
      <button
        type="button"
        className="flex w-full items-center gap-1.5 rounded-md px-1.5 py-1 text-[11px] font-medium text-muted-foreground transition-colors hover:bg-[var(--app-hover)] hover:text-foreground"
        onClick={() => onToggle(groupKey)}
        aria-expanded={isOpen}
      >
        {isOpen ? (
          <ChevronDown className="h-3 w-3 shrink-0" />
        ) : (
          <ChevronRight className="h-3 w-3 shrink-0" />
        )}
        {icon}
        <span className="flex-1 truncate">{label}</span>
        <span className="text-[10px] text-[var(--app-text-dim)]">{count}</span>
      </button>
      {isOpen && <div className="ml-3 flex flex-col gap-0.5">{children}</div>}
    </div>
  );
}

export function ExplorerView() {
  const { t } = useTranslation();
  const { openSchemaPreview, promoteSchemaPreview, openTableData } = useSidebarTabOps();
  const expandedNodes = useExplorerStore((s) => s.expandedNodes);
  const toggleNode = useExplorerStore((s) => s.toggleNode);

  const connections = useConnectionList();
  const statuses = useConnectionModuleStore((s) => s.statuses);
  const explorerConnectionId = useConnectionStore((s) => s.explorerConnectionId);
  const setExplorerConnection = useConnectionStore((s) => s.setExplorerConnection);
  const openConnectionDialog = useRecentStore((s) => s.openConnectionDialog);
  const connect = useConnect();
  const introspect = useIntrospect(explorerConnectionId);

  const handleConnectionClick = (connId: string) => {
    const status = statusOf(statuses, connId);

    if (status === "disconnected" || status === "error") {
      connect.mutate(connId);
      return;
    }

    setExplorerConnection(connId);
    toggleNode(`conn:${connId}`);
  };

  const isConnExpanded = (connId: string) =>
    explorerConnectionId === connId && expandedNodes.includes(`conn:${connId}`);

  return (
    <div className="flex min-h-0 flex-col">
      {/* Search bar */}
      <div className="px-1 pb-2">
        <div className="flex items-center gap-1.5 rounded-md border border-[var(--app-border)] bg-background px-2 py-1">
          <Search className="h-3.5 w-3.5 shrink-0 text-[var(--app-text-dim)]" />
          <input
            type="text"
            placeholder={`${t("shell.sidebar.searchObjects")}...`}
            className="w-full bg-transparent text-xs text-foreground placeholder:text-[var(--app-text-dim)] focus:outline-none"
          />
          <kbd className="shrink-0 text-[9px] text-[var(--app-text-dim)]">⌘K</kbd>
        </div>
      </div>

      <div className="flex flex-col gap-0.5">
        {connections.isLoading && (
          <p className="px-2 py-1 text-xs text-[var(--app-text-dim)]">{t("common.states.loading")}</p>
        )}
        {!connections.isLoading && (connections.data?.length ?? 0) === 0 && (
          <p className="px-2 py-1 text-xs text-[var(--app-text-dim)]">{t("common.states.empty")}</p>
        )}
        {connections.data?.map((conn) => {
          const status = statusOf(statuses, conn.id);
          const expanded = isConnExpanded(conn.id);
          return (
            <div key={conn.id}>
              <ContextMenu>
                <ContextMenuTrigger asChild>
                  <button
                    type="button"
                    className={cn(
                      "flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-[12.5px] text-muted-foreground transition-colors hover:bg-[var(--app-hover)] hover:text-foreground",
                      explorerConnectionId === conn.id && "bg-[var(--app-active)] text-foreground font-medium",
                    )}
                    onClick={() => handleConnectionClick(conn.id)}
                    title={conn.name}
                  >
                    {expanded ? (
                      <ChevronDown className="h-3 w-3 shrink-0" />
                    ) : (
                      <ChevronRight className="h-3 w-3 shrink-0" />
                    )}
                    <StatusDot status={status} />
                    <span className="flex-1 truncate text-left">{conn.name}</span>
                    {conn.group && (
                      <small className="text-[10px] text-[var(--app-text-dim)]">{conn.group}</small>
                    )}
                  </button>
                </ContextMenuTrigger>
                <ContextMenuContent>
                  <ContextMenuItem onClick={() => openConnectionDialog(conn.id)}>
                    {t("shell.sidebar.editConnection")}
                  </ContextMenuItem>
                </ContextMenuContent>
              </ContextMenu>

              {expanded && introspect.isLoading && (
                <p className="ml-7 px-2 py-1 text-xs text-[var(--app-text-dim)]">{t("common.states.loading")}</p>
              )}
              {expanded && introspect.data && (
                <div className="ml-5 flex flex-col gap-0.5 border-l border-[var(--app-border-subtle)] pl-2">
                  {introspect.data.schemas.map((schema) => {
                    const schemaExpanded = expandedNodes.includes(`schema:${conn.id}:${schema.name}`);
                    const tables = introspect.data.tables.filter((tbl) => tbl.schema === schema.name);
                    const views = introspect.data.views.filter((v) => v.schema === schema.name);
                    const Icon = schemaExpanded ? FolderOpen : Folder;
                    return (
                      <div key={schema.name}>
                        <Button
                          type="button"
                          variant="ghost"
                          className="flex h-auto w-full items-center gap-2 justify-start rounded-md px-2 py-1.5 text-xs text-muted-foreground"
                          onClick={() => toggleNode(`schema:${conn.id}:${schema.name}`)}
                          aria-expanded={schemaExpanded}
                        >
                          {schemaExpanded ? (
                            <ChevronDown className="h-3 w-3 shrink-0" />
                          ) : (
                            <ChevronRight className="h-3 w-3 shrink-0" />
                          )}
                          <Icon className="h-3.5 w-3.5 shrink-0 text-primary" />
                          <span className="flex-1 truncate">{schema.name}</span>
                          <span className="text-[10px] text-[var(--app-text-dim)]">
                            {tables.length + views.length}
                          </span>
                        </Button>
                        {schemaExpanded && (
                          <div className="ml-[22px] flex flex-col gap-0.5 border-l border-[var(--app-border-subtle)] pl-2">
                            {tables.length > 0 && (
                              <SchemaObjectGroup
                                groupKey={`schema:${conn.id}:${schema.name}:tables`}
                                label={t("shell.sidebar.tables")}
                                count={tables.length}
                                icon={<Table2 className="h-3 w-3 shrink-0 text-muted-foreground" />}
                                expandedNodes={expandedNodes}
                                onToggle={toggleNode}
                              >
                                {tables.map((table) => {
                                  const qualifiedName = `${schema.name}.${table.name}`;
                                  const driver = getDriverForConnection(conn.id);
                                  const dialect = getSqlDialect(driver);
                                  const countSql = generateCountSQL(dialect, schema.name, table.name);
                                  return (
                                  <ContextMenu key={table.name}>
                                    <ContextMenuTrigger asChild>
                                      <button
                                        type="button"
                                        title={`${schema.name}.${table.name}`}
                                        className="flex w-full items-center gap-2 rounded-md px-2 py-1 text-xs text-muted-foreground transition-colors hover:bg-[var(--app-hover)] hover:text-foreground"
                                        onClick={() => openSchemaPreview(conn.id, schema.name, table.name, "table")}
                                        onDoubleClick={() => promoteSchemaPreview(conn.id, schema.name, table.name)}
                                      >
                                        <Table2 className="h-3 w-3 shrink-0 text-primary" />
                                        <span className="truncate">{table.name}</span>
                                      </button>
                                    </ContextMenuTrigger>
                                    <ContextMenuContent>
                                      <ContextMenuItem onClick={() => openSchemaPreview(conn.id, schema.name, table.name, "table")}>
                                        {t("shell.sidebar.open")}
                                      </ContextMenuItem>
                                      <ContextMenuItem onClick={() => openTableData(conn.id, schema.name, table.name)}>
                                        {t("shell.sidebar.openData")}
                                      </ContextMenuItem>
                                      <ContextMenuSeparator />
                                      <ContextMenuSub>
                                        <ContextMenuSubTrigger>
                                          <Plus className="mr-1.5 h-3 w-3" />
                                          {t("shell.sidebar.newQuery")}
                                        </ContextMenuSubTrigger>
                                        <ContextMenuSubContent>
                                          <ContextMenuItem onClick={() => openNewQueryWithSql(conn.id, schema.name, table.name, dialect.generateSelect({ schema: schema.name, table: table.name, limit: 100 }))}>
                                            SELECT
                                          </ContextMenuItem>
                                          <ContextMenuItem onClick={() => openNewQueryWithSql(conn.id, schema.name, table.name, countSql)}>
                                            COUNT
                                          </ContextMenuItem>
                                          <ContextMenuItem onClick={() => openNewQueryWithSql(conn.id, schema.name, table.name, `INSERT INTO ${dialect.qualify(schema.name, table.name)} ()\nVALUES ();`)}>
                                            INSERT
                                          </ContextMenuItem>
                                          <ContextMenuItem onClick={() => openNewQueryWithSql(conn.id, schema.name, table.name, `UPDATE ${dialect.qualify(schema.name, table.name)}\nSET \nWHERE ;`)}>
                                            UPDATE
                                          </ContextMenuItem>
                                          <ContextMenuItem onClick={() => openNewQueryWithSql(conn.id, schema.name, table.name, `DELETE FROM ${dialect.qualify(schema.name, table.name)}\nWHERE ;`)}>
                                            DELETE
                                          </ContextMenuItem>
                                        </ContextMenuSubContent>
                                      </ContextMenuSub>
                                      <ContextMenuSeparator />
                                      <ContextMenuItem onClick={() => copyToClipboard(table.name)}>
                                        <Copy className="mr-1.5 h-3 w-3" />
                                        {t("shell.sidebar.copyName")}
                                      </ContextMenuItem>
                                      <ContextMenuItem onClick={() => copyToClipboard(qualifiedName)}>
                                        <Copy className="mr-1.5 h-3 w-3" />
                                        {t("shell.sidebar.copyQualifiedName")}
                                      </ContextMenuItem>
                                    </ContextMenuContent>
                                  </ContextMenu>
                                  );
                                })}
                              </SchemaObjectGroup>
                            )}
                            {views.length > 0 && (
                              <SchemaObjectGroup
                                groupKey={`schema:${conn.id}:${schema.name}:views`}
                                label={t("shell.sidebar.views")}
                                count={views.length}
                                icon={<Columns3 className="h-3 w-3 shrink-0 text-muted-foreground" />}
                                expandedNodes={expandedNodes}
                                onToggle={toggleNode}
                              >
                                {views.map((view) => (
                                  <ContextMenu key={view.name}>
                                    <ContextMenuTrigger asChild>
                                      <button
                                        type="button"
                                        title={`${schema.name}.${view.name}`}
                                        className="flex w-full items-center gap-2 rounded-md px-2 py-1 text-xs text-muted-foreground transition-colors hover:bg-[var(--app-hover)] hover:text-foreground"
                                        onClick={() => openSchemaPreview(conn.id, schema.name, view.name, "view")}
                                        onDoubleClick={() => promoteSchemaPreview(conn.id, schema.name, view.name)}
                                      >
                                        <Columns3 className="h-3 w-3 shrink-0 text-primary" />
                                        <span className="truncate">{view.name}</span>
                                      </button>
                                    </ContextMenuTrigger>
                                    <ContextMenuContent>
                                      <ContextMenuItem onClick={() => openTableData(conn.id, schema.name, view.name, "view")}>
                                        {t("shell.sidebar.openData")}
                                      </ContextMenuItem>
                                    </ContextMenuContent>
                                  </ContextMenu>
                                ))}
                              </SchemaObjectGroup>
                            )}
                          </div>
                        )}
                      </div>
                    );
                  })}
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
