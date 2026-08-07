import { useLocation } from "@tanstack/react-router";
import { ChevronRight, Command, Search } from "lucide-react";

import { useTranslation } from "@/commons/locales/useTranslation";
import { useConnectionStore } from "@/commons/stores/connection.store";
import { Button } from "@/components/ui/button";
import { useConnectionList } from "@/modules/connection/queries/connection.queries";

const PAGE_LABEL_KEYS: Record<string, string> = {
  "/connections": "connection.title",
  "/connection-editor": "connection.edit",
  "/query": "query.title",
  "/data": "dataGrid.title",
  "/schema": "schema.title",
  "/users": "userManagement.title",
};

export function Topbar() {
  const location = useLocation();
  const { t } = useTranslation();
  const connections = useConnectionList();
  const activeConnectionId = useConnectionStore((s) => s.activeConnectionId);
  const activeConnection = connections.data?.find((c) => c.id === activeConnectionId) ?? null;
  const pageLabelKey = PAGE_LABEL_KEYS[location.pathname] ?? "";

  return (
    <header
      className="flex items-center border-b border-border bg-card px-4"
      style={{ height: "var(--app-topbar-height)" }}
      role="banner"
    >
      <div className="flex items-center gap-2 text-xs text-muted-foreground">
        <ChevronRight className="h-3.5 w-3.5" />
        <strong className="font-medium text-foreground">
          {activeConnection?.name ?? t("shell.topbar.noConnection")}
        </strong>
        {pageLabelKey && (
          <>
            <span>/</span>
            <span>{t(pageLabelKey)}</span>
          </>
        )}
      </div>
      <div className="ml-auto flex items-center gap-1">
        <Button type="button" variant="ghost" size="icon" className="h-7 w-7" title={t("shell.topbar.commandMenu")}>
          <Command className="h-3.5 w-3.5" />
        </Button>
        <Button type="button" variant="ghost" size="icon" className="h-7 w-7" title={t("shell.topbar.search")}>
          <Search className="h-3.5 w-3.5" />
        </Button>
        <span className="ml-1 grid h-7 w-7 place-items-center rounded-full bg-muted text-[10px] font-semibold text-foreground">
          T
        </span>
      </div>
    </header>
  );
}
