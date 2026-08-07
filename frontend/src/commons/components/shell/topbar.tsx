import { ChevronRight, Command, Search } from "lucide-react";

import { useTranslation } from "@/commons/locales/useTranslation";
import { useCommandStore } from "@/commons/stores/command.store";
import { useConnectionStore } from "@/commons/stores/connection.store";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { Button } from "@/components/ui/button";
import { useConnectionList } from "@/modules/connection/queries/connection.queries";

export function Topbar() {
  const { t } = useTranslation();
  const connections = useConnectionList();
  const explorerConnectionId = useConnectionStore((s) => s.explorerConnectionId);
  const activeConnection = connections.data?.find((c) => c.id === explorerConnectionId) ?? null;
  const activeTab = useWorkspaceStore((s) => {
    if (!s.activeTabId) return null;
    return s.tabs.find((t) => t.id === s.activeTabId) ?? null;
  });

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
        {activeTab && (
          <>
            <span>/</span>
            <span>{activeTab.title}</span>
          </>
        )}
      </div>
      <div className="ml-auto flex items-center gap-1">
        <Button type="button" variant="ghost" size="icon" className="h-7 w-7" title={t("shell.topbar.commandMenu")} onClick={() => useCommandStore.getState().open()}>
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
