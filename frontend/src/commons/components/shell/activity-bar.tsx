import { Link } from "@tanstack/react-router";
import {
  FolderOpen,
  KeyRound,
  PanelLeftClose,
  PanelLeftOpen,
  Search,
  Command,
} from "lucide-react";
import type { ComponentType } from "react";

import { useShellStore, type SidebarView } from "@/commons/stores/shell.store";
import { useTranslation } from "@/commons/locales/useTranslation";
import { Button } from "@/components/ui/button";
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
      className="flex flex-col items-center border-r border-border bg-sidebar py-2"
      style={{ width: "var(--app-activity-bar-width)" }}
    >
      <Link to="/" className="mb-3 grid h-8 w-8 place-items-center rounded-lg bg-primary text-[10px] font-bold text-primary-foreground">
        DB
      </Link>

      <nav className="flex flex-1 flex-col items-center gap-1" role="navigation" aria-label={t("shell.nav.label")}>
        {NAV_ITEMS.map((item) => {
          const isActive = sidebarView === item.viewId;
          const Icon = item.icon;
          const label = t(item.labelKey);
          return (
            <Tooltip key={item.viewId}>
              <TooltipTrigger asChild>
                <Button
                  type="button"
                  variant="ghost"
                  className={cn(
                    "h-9 w-9 p-0 text-muted-foreground hover:bg-[var(--app-hover)] hover:text-foreground",
                    isActive && "bg-[var(--app-active)] text-primary",
                  )}
                  onClick={() => setSidebarView(item.viewId)}
                  aria-current={isActive ? "page" : undefined}
                >
                  <Icon className="h-[18px] w-[18px]" />
                </Button>
              </TooltipTrigger>
              <TooltipContent side="right" sideOffset={8}>
                {label}
              </TooltipContent>
            </Tooltip>
          );
        })}
      </nav>

      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            type="button"
            variant="ghost"
            size="icon"
            className="h-8 w-8 text-muted-foreground hover:bg-[var(--app-hover)]"
            onClick={toggleSidebar}
            aria-label={sidebarCollapsed ? t("shell.expand") : t("shell.collapse")}
          >
            {sidebarCollapsed ? (
              <PanelLeftOpen className="h-4 w-4" />
            ) : (
              <PanelLeftClose className="h-4 w-4" />
            )}
          </Button>
        </TooltipTrigger>
        <TooltipContent side="right" sideOffset={8}>
          {sidebarCollapsed ? t("shell.expand") : t("shell.collapse")}
        </TooltipContent>
      </Tooltip>
    </aside>
  );
}
