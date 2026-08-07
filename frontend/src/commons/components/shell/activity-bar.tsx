import { Link } from "@tanstack/react-router";
import {
  FolderOpen,
  KeyRound,
  PanelLeftClose,
  PanelLeftOpen,
  Command,
  Search,
} from "lucide-react";
import type { ComponentType } from "react";

import { useShellStore, type SidebarView } from "@/commons/stores/shell.store";
import { useTranslation } from "@/commons/locales/useTranslation";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";

interface NavItem {
  viewId: SidebarView;
  labelKey: string;
  icon: ComponentType<{ className?: string }>;
}

const NAV_ITEMS: NavItem[] = [
  { viewId: "explorer", labelKey: "shell.nav.explorer", icon: FolderOpen },
  { viewId: "search", labelKey: "shell.nav.search", icon: Search },
  { viewId: "query-saved", labelKey: "shell.nav.query", icon: Command },
  { viewId: "users", labelKey: "shell.nav.users", icon: KeyRound },
];

export function ActivityBar() {
  const { t } = useTranslation();
  const sidebarCollapsed = useShellStore((s) => s.sidebarCollapsed);
  const toggleSidebar = useShellStore((s) => s.toggleSidebar);
  const sidebarView = useShellStore((s) => s.sidebarView);
  const setSidebarView = useShellStore((s) => s.setSidebarView);

  return (
    <aside
      className="flex flex-col items-center border-r border-[var(--app-border-subtle)] bg-sidebar"
      style={{ width: "var(--app-activity-bar-width)" }}
    >
      {/* Logo */}
      <Link
        to="/"
        className="mb-2 mt-2 grid h-8 w-8 place-items-center rounded-lg bg-primary text-[10px] font-bold text-primary-foreground"
      >
        DB
      </Link>

      {/* Navigation */}
      <nav className="flex flex-1 flex-col items-center gap-0.5 pt-1" role="navigation" aria-label={t("shell.nav.label")}>
        {NAV_ITEMS.map((item) => {
          const isActive = sidebarView === item.viewId;
          const Icon = item.icon;
          const label = t(item.labelKey);
          return (
            <Tooltip key={item.viewId}>
              <TooltipTrigger asChild>
                <button
                  type="button"
                  className={cn(
                    "relative flex h-10 w-10 items-center justify-center rounded-md text-muted-foreground transition-colors duration-100",
                    "hover:bg-[var(--app-hover)] hover:text-foreground",
                    isActive && "text-primary",
                  )}
                  onClick={() => setSidebarView(item.viewId)}
                  aria-current={isActive ? "page" : undefined}
                >
                  {/* Active indicator — 2px left bar */}
                  {isActive && (
                    <span className="absolute left-0 top-1/2 h-5 w-[2px] -translate-y-1/2 rounded-r-sm bg-primary" />
                  )}
                  <Icon className="h-[18px] w-[18px]" />
                </button>
              </TooltipTrigger>
              <TooltipContent side="right" sideOffset={8}>
                {label}
              </TooltipContent>
            </Tooltip>
          );
        })}
      </nav>

      {/* Bottom — collapse toggle */}
      <div className="pb-2">
        <Tooltip>
          <TooltipTrigger asChild>
            <button
              type="button"
              className="flex h-8 w-8 items-center justify-center rounded-md text-muted-foreground transition-colors duration-100 hover:bg-[var(--app-hover)] hover:text-foreground"
              onClick={toggleSidebar}
              aria-label={sidebarCollapsed ? t("shell.expand") : t("shell.collapse")}
            >
              {sidebarCollapsed ? (
                <PanelLeftOpen className="h-4 w-4" />
              ) : (
                <PanelLeftClose className="h-4 w-4" />
              )}
            </button>
          </TooltipTrigger>
          <TooltipContent side="right" sideOffset={8}>
            {sidebarCollapsed ? t("shell.expand") : t("shell.collapse")}
          </TooltipContent>
        </Tooltip>
      </div>
    </aside>
  );
}
