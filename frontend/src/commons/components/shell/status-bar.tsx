import { Database, Wifi, WifiOff } from "lucide-react";

import { useTranslation } from "@/commons/locales/useTranslation";
import { useConnectionStore } from "@/commons/stores/connection.store";
import { cn } from "@/lib/utils";
import { useConnectionList } from "@/modules/connection/queries/connection.queries";
import { useConnectionModuleStore } from "@/modules/connection/state/connection.store";
import { useIntrospect } from "@/modules/schema/queries/schema.queries";

export function StatusBar() {
  const { t } = useTranslation();
  const connections = useConnectionList();
  const explorerConnectionId = useConnectionStore((s) => s.explorerConnectionId);
  const statuses = useConnectionModuleStore((s) => s.statuses);
  const activeConnection = connections.data?.find((c) => c.id === explorerConnectionId) ?? null;
  const introspect = useIntrospect(explorerConnectionId);

  const status = explorerConnectionId ? (statuses[explorerConnectionId] ?? "disconnected") : null;
  const tableCount = introspect.data?.tables.length ?? 0;

  const statusLabel =
    status === "connected"
      ? t("shell.statusbar.connected")
      : status === "connecting"
        ? t("shell.statusbar.connecting")
        : t("shell.statusbar.disconnected");

  return (
    <footer
      className="flex items-center border-t border-border bg-card px-3 text-[10px] text-[var(--app-text-dim)]"
      style={{ height: "var(--app-statusbar-height)" }}
      role="contentinfo"
    >
      {activeConnection ? (
        <>
          <span className="flex items-center gap-1.5">
            {status === "connected" ? (
              <Wifi className="h-3 w-3 text-success" />
            ) : (
              <WifiOff className="h-3 w-3" />
            )}
            <span className={cn(status === "connected" && "text-success")}>{statusLabel}</span>
          </span>
          <span className="mx-2 text-border">|</span>
          <span className="flex items-center gap-1.5">
            <Database className="h-3 w-3" />
            <span>{activeConnection.driver}</span>
          </span>
          <span className="mx-2 text-border">|</span>
          <span>{activeConnection.database}</span>
          <span className="mx-2 text-border">|</span>
          <span>
            {tableCount} {t("shell.statusbar.tables")}
          </span>
        </>
      ) : (
        <span>{t("shell.statusbar.noConnection")}</span>
      )}
      <span className="ml-auto">{t("shell.statusbar.version")}</span>
    </footer>
  );
}
