import { CircleHelp, Settings2 } from "lucide-react";

import { useTranslation } from "@/commons/locales/useTranslation";
import { useShellStore } from "@/commons/stores/shell.store";
import { Button } from "@/components/ui/button";
import { ExplorerView } from "./sidebar-views/explorer-view";
import { QuerySavedView } from "./sidebar-views/query-saved-view";
import { SearchView } from "./sidebar-views/search-view";
import { UsersView } from "./sidebar-views/users-view";

const VIEW_TITLES: Record<string, string> = {
  explorer: "shell.sidebar.explorer",
  search: "shell.sidebar.searchObjects",
  "query-saved": "query.savedQueries",
  users: "userManagement.title",
};

export function Sidebar() {
  const { t } = useTranslation();
  const sidebarCollapsed = useShellStore((s) => s.sidebarCollapsed);
  const sidebarView = useShellStore((s) => s.sidebarView);

  return (
    <aside
      className="flex min-h-0 flex-col overflow-hidden border-r border-border bg-sidebar transition-[width] duration-200 ease-out"
      style={{ width: sidebarCollapsed ? "var(--app-sidebar-collapsed-width)" : "var(--app-sidebar-width)" }}
      aria-label={t("shell.sidebar.label")}
    >
      <div
        className="flex min-h-0 flex-col px-2.5 py-3"
        aria-hidden={sidebarCollapsed}
        inert={sidebarCollapsed ? true : undefined}
        style={{ visibility: sidebarCollapsed ? "hidden" : undefined }}
      >
        <div className="shrink-0 px-2 pb-2">
          <span className="text-[10px] font-semibold uppercase tracking-widest text-[var(--app-text-dim)]">
            {t(VIEW_TITLES[sidebarView] ?? "")}
          </span>
        </div>

        <div className="min-h-0 flex-1 overflow-y-auto">
          {sidebarView === "explorer" && <ExplorerView />}
          {sidebarView === "search" && <SearchView />}
          {sidebarView === "query-saved" && <QuerySavedView />}
          {sidebarView === "users" && <UsersView />}
        </div>

        <div className="shrink-0 border-t border-border bg-sidebar pt-2">
          <Button
            type="button"
            variant="ghost"
            className="flex h-auto w-full items-center gap-2.5 justify-start rounded-md px-2 py-1.5 text-xs text-muted-foreground"
          >
            <Settings2 className="h-3.5 w-3.5 shrink-0 text-primary" />
            <span>{t("shell.sidebar.settings")}</span>
          </Button>
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
