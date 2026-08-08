import { Database, Wifi, WifiOff } from "lucide-react";

import { useTranslation } from "@/commons/locales/useTranslation";
import { useConnectionStore } from "@/commons/stores/connection.store";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { useConnectionList } from "@/modules/connection/queries/connection.queries";
import { useConnectionModuleStore } from "@/modules/connection/state/connection.store";
import { useIntrospect } from "@/modules/schema/queries/schema.queries";

export function StatusBar() {
  const { t } = useTranslation();
  const connections = useConnectionList();
  const explorerConnectionId = useConnectionStore((s) => s.explorerConnectionId);
  const statuses = useConnectionModuleStore((s) => s.statuses);
  const activeTab = useWorkspaceStore((s) => {
    if (!s.activeTabId) return null;
    return s.tabs.find((t) => t.id === s.activeTabId) ?? null;
  });

  const workspaceConnectionId = activeTab?.connectionId ?? explorerConnectionId;
  const activeConnection = connections.data?.find((c) => c.id === workspaceConnectionId) ?? null;
  const introspect = useIntrospect(workspaceConnectionId);

  const status = workspaceConnectionId ? (statuses[workspaceConnectionId] ?? "disconnected") : null;
  const tableCount = introspect.data?.tables.length ?? 0;

  const statusLabel =
    status === "connected"
      ? t("shell.statusbar.connected")
      : status === "connecting"
        ? t("shell.statusbar.connecting")
        : t("shell.statusbar.disconnected");

  return (
    <footer
      className="flex items-center border-t border-[var(--app-border-subtle)] bg-[var(--app-surface-2)] px-3 text-[11px] text-[var(--app-text-dim)]"
      style={{ height: "var(--app-statusbar-height)" }}
      role="contentinfo"
    >
      {/* Left — connection context */}
      {activeConnection && status === "connected" ? (
        <>
          <span className="flex items-center gap-1.5">
            <Wifi className="h-3 w-3 text-[var(--app-success)]" />
            <span className="text-[var(--app-success)]">{statusLabel}</span>
          </span>
          <span className="mx-2 text-[var(--app-border-strong)]">·</span>
          <span className="flex items-center gap-1">
            <Database className="h-3 w-3" />
            <span>{activeConnection.driver}</span>
          </span>
          <span className="mx-2 text-[var(--app-border-strong)]">·</span>
          <span>{activeConnection.database}</span>
          <span className="mx-2 text-[var(--app-border-strong)]">·</span>
          <span>
            {tableCount} {t("shell.statusbar.tables")}
          </span>
        </>
      ) : activeConnection ? (
        <>
          <span className="flex items-center gap-1.5">
            {status === "connecting" ? (
              <span className="h-2.5 w-2.5 animate-pulse rounded-full bg-[var(--app-warning)]" />
            ) : (
              <WifiOff className="h-3 w-3" />
            )}
            <span>{statusLabel}</span>
          </span>
          <span className="mx-2 text-[var(--app-border-strong)]">·</span>
          <span>{activeConnection.name}</span>
        </>
      ) : (
        <span>{t("shell.statusbar.noConnection")}</span>
      )}

      {/* Right — metadata */}
      <span className="ml-auto">{t("shell.statusbar.version")}</span>
    </footer>
  );
}
