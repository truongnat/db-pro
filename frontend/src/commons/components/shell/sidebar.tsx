import { CircleHelp, Plus, Settings2 } from "lucide-react";

import { useTranslation } from "@/commons/locales/useTranslation";
import { useShellStore } from "@/commons/stores/shell.store";
import { useRecentStore } from "@/commons/stores/recent.store";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
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
  const openConnectionDialog = useRecentStore((s) => s.openConnectionDialog);

  return (
    <aside
      className="flex min-h-0 flex-col overflow-hidden border-r border-[var(--app-border-subtle)] bg-sidebar transition-[width] duration-200 ease-out"
      style={{ width: sidebarCollapsed ? "var(--app-sidebar-collapsed-width)" : "var(--app-sidebar-width)" }}
      aria-label={t("shell.sidebar.label")}
    >
      <div
        className="flex min-h-0 flex-col"
        aria-hidden={sidebarCollapsed ? true : undefined}
        inert={sidebarCollapsed ? true : undefined}
        style={{ visibility: sidebarCollapsed ? "hidden" : undefined }}
      >
        {/* Header */}
        <div className="flex shrink-0 items-center justify-between px-3 pt-3 pb-2">
          <span className="text-[11px] font-semibold uppercase tracking-[0.04em] text-[var(--app-text-dim)]">
            {t(VIEW_TITLES[sidebarView] ?? "")}
          </span>
          <Tooltip>
            <TooltipTrigger asChild>
              <button
                type="button"
                className="flex h-5 w-5 items-center justify-center rounded text-muted-foreground transition-colors hover:bg-[var(--app-hover)] hover:text-foreground"
                onClick={() => openConnectionDialog()}
              >
                <Plus className="h-3.5 w-3.5" />
              </button>
            </TooltipTrigger>
            <TooltipContent side="right" sideOffset={4}>{t("connection.new")}</TooltipContent>
          </Tooltip>
        </div>

        {/* Content */}
        <div className="min-h-0 flex-1 overflow-y-auto px-1.5">
          {sidebarView === "explorer" && <ExplorerView />}
          {sidebarView === "search" && <SearchView />}
          {sidebarView === "query-saved" && <QuerySavedView />}
          {sidebarView === "users" && <UsersView />}
        </div>

        {/* Footer */}
        <div className="shrink-0 border-t border-[var(--app-border-subtle)] px-2 pt-2 pb-2">
          <button
            type="button"
            className="flex h-auto w-full items-center gap-2.5 rounded-md px-2 py-1.5 text-xs text-muted-foreground transition-colors hover:bg-[var(--app-hover)] hover:text-foreground"
          >
            <Settings2 className="h-3.5 w-3.5 shrink-0 text-primary" />
            <span>{t("shell.sidebar.settings")}</span>
          </button>
          <button
            type="button"
            className="flex h-auto w-full items-center gap-2.5 rounded-md px-2 py-1.5 text-xs text-muted-foreground transition-colors hover:bg-[var(--app-hover)] hover:text-foreground"
          >
            <CircleHelp className="h-3.5 w-3.5 shrink-0 text-primary" />
            <span>{t("shell.sidebar.help")}</span>
          </button>
        </div>
      </div>
    </aside>
  );
}
