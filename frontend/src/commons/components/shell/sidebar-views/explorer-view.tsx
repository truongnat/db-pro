import {
  ChevronDown,
  ChevronRight,
  Columns3,
  Copy,
  Folder,
  FolderOpen,
  Plus,
  RefreshCw,
  Search,
  Table2,
  Zap,
  ZapOff,
} from "lucide-react";

import { useTranslation } from "@/commons/locales/useTranslation";
import { useConnectionStore } from "@/commons/stores/connection.store";
import { useExplorerStore } from "@/commons/stores/explorer.store";
import { useQuickOpenStore } from "@/commons/stores/quick-open.store";
import { useRecentStore } from "@/commons/stores/recent.store";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { useSidebarTabOps } from "@/commons/hooks/use-sidebar-tab-ops";
import { StatusDot } from "@/commons/components/shell/status-dot";
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
import { isMac } from "@/commons/utils/platform";
import { cn } from "@/lib/utils";
import {
  useConnectionList,
  useConnect,
  useDisconnect,
} from "@/modules/connection/queries/connection.queries";
import { useConnectionModuleStore } from "@/modules/connection/state/connection.store";
import { useIntrospect, refreshIntrospection } from "@/modules/schema/queries/schema.queries";
import { useQueryClient } from "@tanstack/react-query";
import { createQueryTabForObject } from "@/modules/query/controllers/query-workspace.controller";
import { createQueryTab } from "@/commons/factories/tab-factories";
import { getSqlDialect } from "@/modules/query/sql/dialect";
import { generateCountSQL } from "@/modules/query/sql/generators";
import type { DriverType } from "@/modules/connection/types/connection.types";

function statusOf(statuses: Record<string, string>, id: string) {
  return statuses[id] ?? "disconnected";
}

function openNewQueryWithSql(
  connectionId: string,
  schema: string,
  objectName: string,
  sql: string,
) {
  const verb = sql.trim().split(/\s+/)[0]?.toUpperCase() ?? "Query";
  const title = `${verb} \u00B7 ${objectName}`;
  const tab = createQueryTabForObject(connectionId, schema, { title, sql });
  useWorkspaceStore.getState().openTab(tab);
}

function copyToClipboard(text: string) {
  navigator.clipboard.writeText(text);
}

function getDriverForConnection(connectionId: string): DriverType {
  const connections = useConnectionStore.getState().connections;
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

function SchemaObjectGroup({
  groupKey,
  label,
  count,
  icon,
  expandedNodes,
  onToggle,
  children,
}: SchemaObjectGroupProps) {
  const isOpen = expandedNodes.includes(groupKey);
  return (
    <div>
      <button
        type="button"
        className="flex h-[26px] w-full cursor-pointer items-center gap-1.5 rounded-md border-l-2 border-l-transparent px-1.5 text-[12px] font-medium text-[var(--app-text-muted)] transition-colors hover:border-l-primary hover:bg-[var(--app-hover)] hover:text-foreground active:bg-[var(--app-active)]"
        onClick={() => onToggle(groupKey)}
        aria-expanded={isOpen}
      >
        {isOpen ? (
          <ChevronDown className="h-3 w-3 shrink-0" />
        ) : (
          <ChevronRight className="h-3 w-3 shrink-0" />
        )}
        {icon}
        <span className="flex-1 truncate text-left">{label}</span>
        <span className="text-[11px] tabular-nums text-[var(--app-text-dim)]">{count}</span>
      </button>
      {isOpen && <div className="ml-[10px] flex flex-col">{children}</div>}
    </div>
  );
}

export function ExplorerView() {
  const { t } = useTranslation();
  const { openSchemaPreview, openTableData, openObjectStructure, openSchemaWorkspace } =
    useSidebarTabOps();
  const expandedNodes = useExplorerStore((s) => s.expandedNodes);
  const toggleNode = useExplorerStore((s) => s.toggleNode);

  const connections = useConnectionList();
  const statuses = useConnectionModuleStore((s) => s.statuses);
  const explorerConnectionId = useConnectionStore((s) => s.explorerConnectionId);
  const setExplorerConnection = useConnectionStore((s) => s.setExplorerConnection);
  const openConnectionDialog = useRecentStore((s) => s.openConnectionDialog);
  const connect = useConnect();
  const disconnect = useDisconnect();
  const introspect = useIntrospect(explorerConnectionId);
  const queryClient = useQueryClient();

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
    <div className="flex min-h-0 flex-col" onContextMenu={(e) => e.preventDefault()}>
      {/* Quick Open trigger */}
      <div className="px-2 pb-2 pt-1">
        <button
          type="button"
          onClick={() => useQuickOpenStore.getState().open()}
          className="flex h-[28px] w-full cursor-pointer items-center gap-1.5 rounded-md border border-[var(--app-border)] bg-background px-2 transition-colors hover:border-primary/50 hover:bg-[var(--app-hover)]"
        >
          <Search className="h-3.5 w-3.5 shrink-0 text-[var(--app-text-muted)]" />
          <span className="w-full text-left text-[13px] text-[var(--app-text-dim)]">{`${t("shell.sidebar.searchObjects")}...`}</span>
          <kbd className="shrink-0 text-[11px] text-[var(--app-text-dim)]">
            {isMac ? "⌘P" : "Ctrl+P"}
          </kbd>
        </button>
      </div>

      <div className="flex flex-col gap-0.5">
        {connections.isLoading && (
          <p className="px-2 py-1 text-[11px] text-[var(--app-text-dim)]">
            {t("common.states.loading")}
          </p>
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
                      "flex w-full cursor-pointer items-center gap-2 rounded-md border-l-2 border-l-transparent px-2 py-[6px] text-[13px] transition-colors hover:border-l-primary hover:bg-[var(--app-hover)] hover:text-foreground active:bg-[var(--app-active)]",
                      explorerConnectionId === conn.id &&
                        "bg-[var(--app-active)] text-foreground font-semibold",
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
                      <small className="text-[11px] text-[var(--app-text-dim)]">{conn.group}</small>
                    )}
                  </button>
                </ContextMenuTrigger>
                <ContextMenuContent>
                  {status === "connected" ? (
                    <ContextMenuItem onClick={() => disconnect.mutate(conn.id)}>
                      <ZapOff className="mr-1.5 h-3 w-3" />
                      {t("common.actions.disconnect")}
                    </ContextMenuItem>
                  ) : (
                    <ContextMenuItem onClick={() => connect.mutate(conn.id)}>
                      <Zap className="mr-1.5 h-3 w-3" />
                      {t("common.actions.connect")}
                    </ContextMenuItem>
                  )}
                  <ContextMenuSeparator />
                  <ContextMenuItem onClick={() => openConnectionDialog(conn.id)}>
                    {t("shell.sidebar.editConnection")}
                  </ContextMenuItem>
                  <ContextMenuSeparator />
                  <ContextMenuItem
                    onClick={() => {
                      const tab = createQueryTab(conn.id);
                      useWorkspaceStore.getState().openTab(tab);
                    }}
                  >
                    <Plus className="mr-1.5 h-3 w-3" />
                    {t("shell.sidebar.newQuery")}
                  </ContextMenuItem>
                  <ContextMenuItem onClick={() => refreshIntrospection(queryClient, conn.id)}>
                    <RefreshCw className="mr-1.5 h-3 w-3" />
                    {t("common.actions.refresh")}
                  </ContextMenuItem>
                  <ContextMenuItem onClick={() => copyToClipboard(conn.name)}>
                    <Copy className="mr-1.5 h-3 w-3" />
                    {t("shell.sidebar.copyConnectionName")}
                  </ContextMenuItem>
                </ContextMenuContent>
              </ContextMenu>

              {expanded && introspect.isLoading && (
                <div className="ml-7 flex flex-col gap-1 px-2 py-1">
                  <div className="h-[26px] animate-pulse rounded bg-[var(--app-hover)]" />
                  <div className="h-[26px] animate-pulse rounded bg-[var(--app-hover)]" />
                  <div className="h-[26px] animate-pulse rounded bg-[var(--app-hover)]" />
                </div>
              )}
              {expanded && introspect.isError && (
                <div className="ml-7 flex flex-col gap-1 px-2 py-2">
                  <p className="text-[12px] text-[var(--app-danger)]">{t("common.states.error")}</p>
                  <button
                    type="button"
                    className="flex cursor-pointer items-center gap-1.5 text-[12px] text-[var(--app-primary)] hover:underline"
                    onClick={() => introspect.refetch()}
                  >
                    <RefreshCw className="h-3 w-3" />
                    {t("common.actions.retry")}
                  </button>
                </div>
              )}
              {expanded && introspect.data && (
                <div className="ml-4 flex flex-col gap-0.5 pl-3">
                  {introspect.data.schemas.map((schema) => {
                    const schemaExpanded = expandedNodes.includes(
                      `schema:${conn.id}:${schema.name}`,
                    );
                    const tables = introspect.data.tables.filter(
                      (tbl) => tbl.schema === schema.name,
                    );
                    const views = introspect.data.views.filter((v) => v.schema === schema.name);
                    const Icon = schemaExpanded ? FolderOpen : Folder;
                    return (
                      <div key={schema.name}>
                        <ContextMenu>
                          <ContextMenuTrigger asChild>
                            <button
                              type="button"
                              className="flex h-[28px] w-full cursor-pointer items-center gap-1.5 rounded-md border-l-2 border-l-transparent px-2 text-left text-[13px] font-medium text-foreground transition-colors hover:border-l-primary hover:bg-[var(--app-hover)] active:bg-[var(--app-active)]"
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
                              <span className="text-[11px] tabular-nums text-[var(--app-text-dim)]">
                                {tables.length + views.length}
                              </span>
                            </button>
                          </ContextMenuTrigger>
                          <ContextMenuContent>
                            <ContextMenuItem
                              onClick={() => openSchemaWorkspace(conn.id, schema.name)}
                            >
                              {t("schemaWorkspace.actions.openDiagram")}
                            </ContextMenuItem>
                            <ContextMenuSeparator />
                            <ContextMenuItem
                              onClick={() => navigator.clipboard.writeText(schema.name)}
                            >
                              {t("shell.sidebar.copyName")}
                            </ContextMenuItem>
                          </ContextMenuContent>
                        </ContextMenu>
                        {schemaExpanded && (
                          <div className="ml-[10px] flex flex-col gap-0.5">
                            {tables.length > 0 && (
                              <SchemaObjectGroup
                                groupKey={`schema:${conn.id}:${schema.name}:tables`}
                                label={t("shell.sidebar.tables")}
                                count={tables.length}
                                icon={
                                  <Table2 className="h-3 w-3 shrink-0 text-[var(--app-text-muted)]" />
                                }
                                expandedNodes={expandedNodes}
                                onToggle={toggleNode}
                              >
                                {tables.map((table) => {
                                  const qualifiedName = `${schema.name}.${table.name}`;
                                  const driver = getDriverForConnection(conn.id);
                                  const dialect = getSqlDialect(driver);
                                  const countSql = generateCountSQL(
                                    dialect,
                                    schema.name,
                                    table.name,
                                  );
                                  return (
                                    <ContextMenu key={table.name}>
                                      <ContextMenuTrigger asChild>
                                        <button
                                          type="button"
                                          title={`${schema.name}.${table.name}`}
                                          className="group flex h-[26px] w-full cursor-pointer items-center gap-2 rounded-md border-l-2 border-l-transparent px-2 text-left text-[13px] text-[var(--app-text-muted)] transition-colors hover:border-l-primary hover:bg-[var(--app-hover)] hover:text-foreground active:bg-[var(--app-active)]"
                                          onClick={() =>
                                            openSchemaPreview(
                                              conn.id,
                                              schema.name,
                                              table.name,
                                              "table",
                                            )
                                          }
                                          onDoubleClick={() =>
                                            openTableData(conn.id, schema.name, table.name, "table")
                                          }
                                        >
                                          <Table2 className="h-3.5 w-3.5 shrink-0 text-[var(--app-text-muted)]" />
                                          <span className="flex-1 truncate">{table.name}</span>
                                        </button>
                                      </ContextMenuTrigger>
                                      <ContextMenuContent>
                                        <ContextMenuItem
                                          onClick={() =>
                                            openTableData(conn.id, schema.name, table.name)
                                          }
                                        >
                                          {t("shell.sidebar.openData")}
                                        </ContextMenuItem>
                                        <ContextMenuItem
                                          onClick={() =>
                                            openObjectStructure(
                                              conn.id,
                                              schema.name,
                                              table.name,
                                              "table",
                                            )
                                          }
                                        >
                                          {t("shell.sidebar.openStructure")}
                                        </ContextMenuItem>
                                        <ContextMenuSeparator />
                                        <ContextMenuSub>
                                          <ContextMenuSubTrigger>
                                            <Plus className="mr-1.5 h-3 w-3" />
                                            {t("shell.sidebar.newQuery")}
                                          </ContextMenuSubTrigger>
                                          <ContextMenuSubContent>
                                            <ContextMenuItem
                                              onClick={() =>
                                                openNewQueryWithSql(
                                                  conn.id,
                                                  schema.name,
                                                  table.name,
                                                  dialect.generateSelect({
                                                    schema: schema.name,
                                                    table: table.name,
                                                    limit: 100,
                                                  }),
                                                )
                                              }
                                            >
                                              SELECT
                                            </ContextMenuItem>
                                            <ContextMenuItem
                                              onClick={() =>
                                                openNewQueryWithSql(
                                                  conn.id,
                                                  schema.name,
                                                  table.name,
                                                  countSql,
                                                )
                                              }
                                            >
                                              COUNT
                                            </ContextMenuItem>
                                            <ContextMenuItem
                                              onClick={() =>
                                                openNewQueryWithSql(
                                                  conn.id,
                                                  schema.name,
                                                  table.name,
                                                  `INSERT INTO ${dialect.qualify(schema.name, table.name)} ()\nVALUES ();`,
                                                )
                                              }
                                            >
                                              INSERT
                                            </ContextMenuItem>
                                            <ContextMenuItem
                                              onClick={() =>
                                                openNewQueryWithSql(
                                                  conn.id,
                                                  schema.name,
                                                  table.name,
                                                  `UPDATE ${dialect.qualify(schema.name, table.name)}\nSET \nWHERE ;`,
                                                )
                                              }
                                            >
                                              UPDATE
                                            </ContextMenuItem>
                                            <ContextMenuItem
                                              onClick={() =>
                                                openNewQueryWithSql(
                                                  conn.id,
                                                  schema.name,
                                                  table.name,
                                                  `DELETE FROM ${dialect.qualify(schema.name, table.name)}\nWHERE ;`,
                                                )
                                              }
                                            >
                                              DELETE
                                            </ContextMenuItem>
                                          </ContextMenuSubContent>
                                        </ContextMenuSub>
                                        <ContextMenuSeparator />
                                        <ContextMenuItem
                                          onClick={() => copyToClipboard(table.name)}
                                        >
                                          <Copy className="mr-1.5 h-3 w-3" />
                                          {t("shell.sidebar.copyName")}
                                        </ContextMenuItem>
                                        <ContextMenuItem
                                          onClick={() => copyToClipboard(qualifiedName)}
                                        >
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
                                icon={
                                  <Columns3 className="h-3 w-3 shrink-0 text-[var(--app-text-muted)]" />
                                }
                                expandedNodes={expandedNodes}
                                onToggle={toggleNode}
                              >
                                {views.map((view) => (
                                  <ContextMenu key={view.name}>
                                    <ContextMenuTrigger asChild>
                                      <button
                                        type="button"
                                        title={`${schema.name}.${view.name}`}
                                        className="group flex h-[26px] w-full cursor-pointer items-center gap-2 rounded-md border-l-2 border-l-transparent px-2 text-left text-[13px] text-[var(--app-text-muted)] transition-colors hover:border-l-primary hover:bg-[var(--app-hover)] hover:text-foreground active:bg-[var(--app-active)]"
                                        onClick={() =>
                                          openSchemaPreview(conn.id, schema.name, view.name, "view")
                                        }
                                        onDoubleClick={() =>
                                          openTableData(conn.id, schema.name, view.name, "view")
                                        }
                                      >
                                        <Columns3 className="h-3.5 w-3.5 shrink-0 text-[var(--app-text-muted)]" />
                                        <span className="flex-1 truncate">{view.name}</span>
                                      </button>
                                    </ContextMenuTrigger>
                                    <ContextMenuContent>
                                      <ContextMenuItem
                                        onClick={() =>
                                          openTableData(conn.id, schema.name, view.name, "view")
                                        }
                                      >
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
