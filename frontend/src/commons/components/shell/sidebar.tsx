import { Link } from "@tanstack/react-router";
import {
  ChevronDown,
  ChevronRight,
  CircleHelp,
  Clock3,
  Columns3,
  Folder,
  FolderOpen,
  Plus,
  Settings2,
  Table2,
} from "lucide-react";
import { useState } from "react";

import { useTranslation } from "@/commons/locales/useTranslation";
import { useConnectionStore } from "@/commons/stores/connection.store";
import { useShellStore } from "@/commons/stores/shell.store";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { useConnectionList } from "@/modules/connection/queries/connection.queries";
import { useConnectionModuleStore } from "@/modules/connection/state/connection.store";
import { useIntrospect } from "@/modules/schema/queries/schema.queries";
import { useSchemaModuleStore } from "@/modules/schema/state/schema.store";

function statusOf(statuses: Record<string, string>, id: string) {
  return statuses[id] ?? "disconnected";
}

function StatusDot({ status }: { status: string }) {
  const className =
    status === "connected"
      ? "bg-success shadow-[0_0_0_3px_rgba(34,197,94,0.15)]"
      : status === "connecting"
        ? "bg-warning shadow-[0_0_0_3px_rgba(229,195,106,0.15)]"
        : status === "error"
          ? "bg-destructive"
          : "bg-muted-foreground";
  return <span className={cn("h-[7px] w-[7px] shrink-0 rounded-full", className)} />;
}

export function Sidebar() {
  const [expandedSchemas, setExpandedSchemas] = useState<Set<string>>(new Set());
  const { t } = useTranslation();
  const sidebarCollapsed = useShellStore((s) => s.sidebarCollapsed);

  const connections = useConnectionList();
  const statuses = useConnectionModuleStore((s) => s.statuses);
  const activeConnectionId = useConnectionStore((s) => s.activeConnectionId);
  const introspect = useIntrospect(activeConnectionId);

  const toggleSchema = (schema: string) =>
    setExpandedSchemas((prev) => {
      const next = new Set(prev);
      if (next.has(schema)) next.delete(schema);
      else next.add(schema);
      return next;
    });

  return (
    <aside
      className="flex min-h-0 flex-col overflow-hidden border-r border-border bg-sidebar transition-[width] duration-200 ease-out"
      style={{ width: sidebarCollapsed ? "var(--app-sidebar-collapsed-width)" : "var(--app-sidebar-width)" }}
      aria-label={t("shell.sidebar.label")}
    >
      <div
        className="flex min-h-0 flex-col overflow-y-auto px-2.5 py-3"
        aria-hidden={sidebarCollapsed}
        inert={sidebarCollapsed ? true : undefined}
        style={{ visibility: sidebarCollapsed ? "hidden" : undefined }}
      >
        <ConnectionsSection
          connections={connections.data}
          isLoading={connections.isLoading}
          statuses={statuses}
          activeConnectionId={activeConnectionId}
          t={t}
        />

        <div className="mt-4">
          <div className="px-2 pb-2">
            <span className="text-[10px] font-semibold uppercase tracking-widest text-[var(--app-text-dim)]">
              {t("shell.sidebar.schemaExplorer")}
            </span>
          </div>
          {!activeConnectionId && (
            <p className="px-2 py-1 text-xs text-[var(--app-text-dim)]">{t("shell.sidebar.connectFirst")}</p>
          )}
          {activeConnectionId && introspect.isLoading && (
            <p className="px-2 py-1 text-xs text-[var(--app-text-dim)]">{t("common.states.loading")}</p>
          )}
          {activeConnectionId && introspect.data && (
            <div className="flex flex-col gap-0.5">
              {introspect.data.schemas.map((schema) => {
                const expanded = expandedSchemas.has(schema.name);
                const tables = introspect.data.tables.filter((tbl) => tbl.schema === schema.name);
                const views = introspect.data.views.filter((v) => v.schema === schema.name);
                const Icon = expanded ? FolderOpen : Folder;
                return (
                  <div key={schema.name}>
                    <Button
                      type="button"
                      variant="ghost"
                      className="flex h-auto w-full items-center gap-2 justify-start rounded-md px-2 py-1.5 text-xs text-muted-foreground"
                      onClick={() => toggleSchema(schema.name)}
                      aria-expanded={expanded}
                    >
                      {expanded ? (
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
                    {expanded && (
                      <div className="ml-[22px] flex flex-col gap-0.5 border-l border-border pl-2">
                        {tables.map((table) => (
                          <Link
                            key={table.name}
                            to="/schema"
                            className="flex items-center gap-2 rounded-md px-2 py-1 text-xs text-muted-foreground transition-colors hover:bg-[var(--app-hover)] hover:text-foreground"
                            onClick={() => useSchemaModuleStore.getState().setSelectedTable(schema.name, table.name, "table")}
                          >
                            <Table2 className="h-3 w-3 shrink-0 text-primary" />
                            <span className="truncate">{table.name}</span>
                          </Link>
                        ))}
                        {views.map((view) => (
                          <Link
                            key={view.name}
                            to="/schema"
                            className="flex items-center gap-2 rounded-md px-2 py-1 text-xs text-muted-foreground transition-colors hover:bg-[var(--app-hover)] hover:text-foreground"
                            onClick={() => useSchemaModuleStore.getState().setSelectedTable(schema.name, view.name, "view")}
                          >
                            <Columns3 className="h-3 w-3 shrink-0 text-primary" />
                            <span className="truncate">{view.name}</span>
                          </Link>
                        ))}
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          )}
        </div>

        <div className="mt-auto flex flex-col gap-0.5 border-t border-border pt-2.5">
          {activeConnectionId && (
            <div className="flex items-center gap-2 rounded-md px-2 py-1.5 text-xs text-[var(--app-text-dim)]">
              <Clock3 className="h-3 w-3 shrink-0" />
              <span className="truncate">{t("shell.sidebar.noRecentQueries")}</span>
            </div>
          )}
          <Link
            to="/connections"
            className="flex items-center gap-2.5 rounded-md px-2 py-1.5 text-xs text-muted-foreground transition-colors hover:bg-[var(--app-hover)] hover:text-foreground"
          >
            <Settings2 className="h-3.5 w-3.5 shrink-0 text-primary" />
            <span>{t("shell.sidebar.settings")}</span>
          </Link>
          <Button
            type="button"
            variant="ghost"
            className="flex h-auto w-full items-center gap-2.5 justify-start rounded-md px-2 py-1.5 text-xs text-muted-foreground"
          >
            <CircleHelp className="h-3.5 w-3.5 shrink-0 text-primary" />
            <span>{t("shell.sidebar.help")}</span>
          </Button>
        </div>
      </div>
    </aside>
  );
}

function ConnectionsSection({
  connections,
  isLoading,
  statuses,
  activeConnectionId,
  t,
}: {
  connections: ReturnType<typeof useConnectionList>["data"];
  isLoading: boolean;
  statuses: Record<string, string>;
  activeConnectionId: string | null;
  t: (key: string) => string;
}) {
  return (
    <div>
      <div className="flex items-center justify-between px-2 pb-2">
        <span className="text-[10px] font-semibold uppercase tracking-widest text-[var(--app-text-dim)]">
          {t("shell.sidebar.connections")}
        </span>
        <Link to="/connections">
          <Button type="button" variant="ghost" size="icon" className="h-5 w-5 text-muted-foreground">
            <Plus className="h-3.5 w-3.5" />
          </Button>
        </Link>
      </div>
      <div className="flex flex-col gap-0.5">
        {isLoading && (
          <p className="px-2 py-1 text-xs text-[var(--app-text-dim)]">{t("common.states.loading")}</p>
        )}
        {!isLoading && (connections?.length ?? 0) === 0 && (
          <p className="px-2 py-1 text-xs text-[var(--app-text-dim)]">{t("common.states.empty")}</p>
        )}
        {connections?.map((conn) => {
          const isActive = conn.id === activeConnectionId;
          return (
            <Link
              key={conn.id}
              to="/connections"
              className={cn(
                "flex items-center gap-2.5 rounded-md px-2 py-1.5 text-xs text-muted-foreground transition-colors hover:bg-[var(--app-hover)] hover:text-foreground",
                isActive && "bg-[var(--app-active)] text-foreground",
              )}
            >
              <StatusDot status={statusOf(statuses, conn.id)} />
              <span className="flex-1 truncate">{conn.name}</span>
              {conn.group && <small className="text-[10px] text-[var(--app-text-dim)]">{conn.group}</small>}
            </Link>
          );
        })}
      </div>
    </div>
  );
}
