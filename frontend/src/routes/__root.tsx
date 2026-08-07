import { createRootRoute, Link, Outlet, useLocation } from "@tanstack/react-router";
import {
  ChevronDown,
  ChevronRight,
  CircleHelp,
  Clock3,
  Columns3,
  Command,
  Database,
  Folder,
  FolderOpen,
  KeyRound,
  Plus,
  RefreshCw,
  Search,
  Settings2,
  Table2,
} from "lucide-react";
import { useState } from "react";

import { useConnectionStore } from "@/commons/stores/connection.store";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { useConnectionList } from "@/modules/connection/queries/connection.queries";
import { useConnectionModuleStore } from "@/modules/connection/state/connection.store";
import { useIntrospect } from "@/modules/schema/queries/schema.queries";
import { useSchemaModuleStore } from "@/modules/schema/state/schema.store";

export const Route = createRootRoute({
  component: RootLayout,
});

const NAV_ITEMS = [
  { to: "/connections", label: "Connections", icon: Database },
  { to: "/query", label: "Query", icon: Command },
  { to: "/data", label: "Data", icon: Table2 },
  { to: "/schema", label: "Schema", icon: FolderOpen },
  { to: "/users", label: "Users", icon: KeyRound },
] as const;

const PAGE_LABELS: Record<string, string> = {
  "/connections": "Connections",
  "/query": "Query",
  "/data": "Data",
  "/schema": "Schema",
  "/users": "Users",
};

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

function RootLayout() {
  const location = useLocation();
  const [expandedSchemas, setExpandedSchemas] = useState<Set<string>>(new Set());

  const connections = useConnectionList();
  const statuses = useConnectionModuleStore((s) => s.statuses);
  const activeConnectionId = useConnectionStore((s) => s.activeConnectionId);
  const introspect = useIntrospect(activeConnectionId);

  const selectedSchema = useSchemaModuleStore((s) => s.selectedSchema);
  const selectedTable = useSchemaModuleStore((s) => s.selectedTable);

  const activeConnection = connections.data?.find((c) => c.id === activeConnectionId) ?? null;
  const pageLabel = PAGE_LABELS[location.pathname] ?? "";

  // TODO: Auto-reconnect disabled temporarily due to hooks ordering issue
  // const autoConnectAttempted = useRef(false);
  // useEffect(() => {
  //   if (autoConnectAttempted.current || !activeConnectionId) return;
  //   autoConnectAttempted.current = true;
  //
  //   const connService = container.resolve<IConnectionService>(SERVICE_NAMES.CONNECTION_SERVICE);
  //   const setStatus = useConnectionModuleStore.getState().setStatus;
  //
  //   setStatus(activeConnectionId, "connecting");
  //   connService.connect(activeConnectionId).then(
  //     () => setStatus(activeConnectionId, "connected"),
  //     () => {
  //       setStatus(activeConnectionId, "disconnected");
  //       useConnectionStore.getState().setActiveConnection(null);
  //     },
  //   );
  // }, [activeConnectionId]);

  const toggleSchema = (schema: string) =>
    setExpandedSchemas((prev) => {
      const next = new Set(prev);
      if (next.has(schema)) {
        next.delete(schema);
      } else {
        next.add(schema);
      }
      return next;
    });

  return (
    <div className="grid h-screen grid-cols-[230px_1fr] grid-rows-[50px_1fr] overflow-hidden">
      <header className="col-span-2 flex items-center border-b border-border bg-card px-4">
        <div className="flex w-[198px] items-center gap-2.5">
          <span className="grid h-6 w-6 place-items-center rounded-md bg-primary text-[10px] font-bold text-primary-foreground">
            DB
          </span>
          <span className="text-sm font-semibold text-foreground">DB Pro</span>
        </div>
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          <ChevronRight className="h-3.5 w-3.5" />
          <strong className="font-medium text-foreground">
            {activeConnection?.name ?? "No connection"}
          </strong>
          {pageLabel && (
            <>
              <span>/</span>
              <span>{pageLabel}</span>
            </>
          )}
        </div>
        <div className="ml-auto flex items-center gap-1">
          <Button type="button" variant="ghost" size="icon" className="h-7 w-7" title="Command menu">
            <Command className="h-3.5 w-3.5" />
          </Button>
          <Button type="button" variant="ghost" size="icon" className="h-7 w-7" title="Search">
            <Search className="h-3.5 w-3.5" />
          </Button>
          <span className="ml-1 grid h-7 w-7 place-items-center rounded-full bg-muted text-[10px] font-semibold text-foreground">
            T
          </span>
        </div>
      </header>

      <aside className="flex min-h-0 flex-col overflow-y-auto border-r border-border bg-card px-2.5 py-3.5">
        <div className="flex items-center justify-between px-2 pb-2">
          <span className="text-[10px] font-semibold uppercase tracking-widest text-muted-foreground">
            Connections
          </span>
          <Link to="/connections">
            <Button type="button" variant="ghost" size="icon" className="h-5 w-5 text-muted-foreground">
              <Plus className="h-3.5 w-3.5" />
            </Button>
          </Link>
        </div>
        <div className="flex flex-col gap-0.5">
          {connections.isLoading && (
            <p className="px-2 py-1 text-xs text-muted-foreground">Loading…</p>
          )}
          {!connections.isLoading && (connections.data?.length ?? 0) === 0 && (
            <p className="px-2 py-1 text-xs text-muted-foreground">No connections</p>
          )}
          {connections.data?.map((conn) => {
            const isActive = conn.id === activeConnectionId;
            return (
              <Link
                key={conn.id}
                to="/connections"
                className={cn(
                  "flex items-center gap-2.5 rounded-md px-2 py-1.5 text-xs text-muted-foreground transition-colors hover:bg-muted hover:text-foreground",
                  isActive && "bg-muted text-foreground",
                )}
              >
                <StatusDot status={statusOf(statuses, conn.id)} />
                <span className="flex-1 truncate">{conn.name}</span>
                {conn.group && <small className="text-[10px] text-muted-foreground">{conn.group}</small>}
              </Link>
            );
          })}
        </div>

        <div className="mt-4">
          <div className="px-2 pb-2">
            <span className="text-[10px] font-semibold uppercase tracking-widest text-muted-foreground">
              Navigation
            </span>
          </div>
          <div className="flex flex-col gap-0.5">
            {NAV_ITEMS.map((item) => {
              const isActive = location.pathname.startsWith(item.to);
              const Icon = item.icon;
              return (
                <Link
                  key={item.to}
                  to={item.to}
                  className={cn(
                    "flex items-center gap-2.5 rounded-md px-2 py-1.5 text-xs text-muted-foreground transition-colors hover:bg-muted hover:text-foreground",
                    isActive && "bg-muted text-foreground",
                  )}
                >
                  <Icon className="h-3.5 w-3.5 shrink-0 text-primary" />
                  <span>{item.label}</span>
                </Link>
              );
            })}
          </div>
        </div>

        <div className="mt-4">
          <div className="flex items-center justify-between px-2 pb-2">
            <span className="text-[10px] font-semibold uppercase tracking-widest text-muted-foreground">
              Schema explorer
            </span>
            <Button type="button" variant="ghost" size="icon" className="h-5 w-5 text-muted-foreground" title="Refresh">
              <RefreshCw className="h-3 w-3" />
            </Button>
          </div>
          {!activeConnectionId && (
            <p className="px-2 py-1 text-xs text-muted-foreground">Connect to a database</p>
          )}
          {activeConnectionId && introspect.isLoading && (
            <p className="px-2 py-1 text-xs text-muted-foreground">Loading…</p>
          )}
          {activeConnectionId && introspect.data && (
            <div className="flex flex-col gap-0.5">
              {introspect.data.schemas.map((schema) => {
                const expanded = expandedSchemas.has(schema.name);
                const tables = introspect.data.tables.filter((t) => t.schema === schema.name);
                const views = introspect.data.views.filter((v) => v.schema === schema.name);
                const Icon = expanded ? FolderOpen : Folder;
                return (
                  <div key={schema.name}>
                    <button
                      type="button"
                      className="flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-xs text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
                      onClick={() => toggleSchema(schema.name)}
                    >
                      {expanded ? (
                        <ChevronDown className="h-3 w-3 shrink-0" />
                      ) : (
                        <ChevronRight className="h-3 w-3 shrink-0" />
                      )}
                      <Icon className="h-3.5 w-3.5 shrink-0 text-primary" />
                      <span className="flex-1 truncate">{schema.name}</span>
                      <span className="text-[10px] text-muted-foreground">
                        {tables.length + views.length}
                      </span>
                    </button>
                    {expanded && (
                      <div className="ml-[22px] flex flex-col gap-0.5 border-l border-border pl-2">
                        {tables.map((table) => (
                          <Link
                            key={table.name}
                            to="/schema"
                            className="flex items-center gap-2 rounded-md px-2 py-1 text-xs text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
                          >
                            <Table2 className="h-3 w-3 shrink-0 text-primary" />
                            <span className="truncate">{table.name}</span>
                          </Link>
                        ))}
                        {views.map((view) => (
                          <Link
                            key={view.name}
                            to="/schema"
                            className="flex items-center gap-2 rounded-md px-2 py-1 text-xs text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
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
          <Link
            to="/connections"
            className="flex items-center gap-2.5 rounded-md px-2 py-1.5 text-xs text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
          >
            <Settings2 className="h-3.5 w-3.5 shrink-0 text-primary" />
            <span>Settings</span>
          </Link>
          <button
            type="button"
            className="flex items-center gap-2.5 rounded-md px-2 py-1.5 text-xs text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
          >
            <CircleHelp className="h-3.5 w-3.5 shrink-0 text-primary" />
            <span>Help</span>
          </button>
        </div>
      </aside>

      <main className="grid min-h-0 min-w-0 grid-cols-[minmax(0,1fr)_270px]">
        <section className="flex min-h-0 min-w-0 flex-col overflow-y-auto">
          <Outlet />
        </section>

        <aside className="hidden overflow-y-auto border-l border-border bg-card md:block">
          {activeConnection ? (
            <>
              <div className="border-b border-border px-4 pb-3 pt-4">
                <small className="text-[9px] uppercase tracking-widest text-muted-foreground">
                  Connection details
                </small>
                <h2 className="mt-1.5 text-sm font-semibold text-foreground">{activeConnection.name}</h2>
                <p className="mt-1 text-xs leading-5 text-muted-foreground">
                  {activeConnection.driver} · {activeConnection.host}:{activeConnection.port}
                  <br />
                  {activeConnection.database}
                </p>
              </div>

              <section className="px-4 pt-3.5">
                <h3 className="mb-2 text-[10px] font-semibold uppercase tracking-widest text-muted-foreground">
                  Overview
                </h3>
                <InfoRow label="Status">
                  <span
                    className={cn(
                      "font-medium",
                      statusOf(statuses, activeConnection.id) === "connected"
                        ? "text-success"
                        : "text-muted-foreground",
                    )}
                  >
                    {statusOf(statuses, activeConnection.id) === "connected"
                      ? "Connected"
                      : statusOf(statuses, activeConnection.id) === "connecting"
                        ? "Connecting"
                        : statusOf(statuses, activeConnection.id) === "error"
                          ? "Error"
                          : "Disconnected"}
                  </span>
                </InfoRow>
                <InfoRow label="Driver">{activeConnection.driver}</InfoRow>
                <InfoRow label="Tables">
                  {introspect.data ? String(introspect.data.tables.length) : "—"}
                </InfoRow>
              </section>

              <section className="px-4 pt-3.5">
                <h3 className="mb-2 text-[10px] font-semibold uppercase tracking-widest text-muted-foreground">
                  Selected table
                </h3>
                {selectedSchema && selectedTable ? (
                  <div className="overflow-hidden rounded-md border border-border bg-muted">
                    <div className="flex items-center gap-2 border-b border-border px-2.5 py-1.5 text-xs font-medium text-foreground">
                      <Table2 className="h-3 w-3 shrink-0 text-primary" />
                      <span className="truncate">{selectedSchema}.{selectedTable}</span>
                    </div>
                  </div>
                ) : (
                  <p className="text-xs text-muted-foreground">No table selected</p>
                )}
              </section>

              <section className="px-4 pt-3.5">
                <h3 className="mb-2 text-[10px] font-semibold uppercase tracking-widest text-muted-foreground">
                  Recent queries
                </h3>
                <div className="flex items-center gap-2 py-1.5 text-xs text-muted-foreground">
                  <Clock3 className="h-3 w-3 shrink-0" />
                  <span>No recent queries</span>
                </div>
              </section>
            </>
          ) : (
            <div className="px-4 pt-4 text-xs text-muted-foreground">
              Connect to a database to see details
            </div>
          )}
        </aside>
      </main>
    </div>
  );
}

function InfoRow({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex items-center justify-between border-b border-border py-2 text-xs text-muted-foreground">
      <span>{label}</span>
      <strong className="font-medium text-foreground">{children}</strong>
    </div>
  );
}
