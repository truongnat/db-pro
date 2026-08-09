import { Link } from "@tanstack/react-router";
import {
  FolderOpen,
  PanelLeftClose,
  PanelLeftOpen,
  Search,
  Sparkles,
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
  // Saved Queries hidden for v0.1 — no real save/load workflow yet.
  // { viewId: "query-saved", labelKey: "shell.nav.query", icon: Command },
  // Users module hidden for v0.1 — no real workbench behind it yet.
  // { viewId: "users", labelKey: "shell.nav.users", icon: KeyRound },
];

export function ActivityBar() {
  const { t } = useTranslation();
  const sidebarCollapsed = useShellStore((s) => s.sidebarCollapsed);
  const toggleSidebar = useShellStore((s) => s.toggleSidebar);
  const sidebarView = useShellStore((s) => s.sidebarView);
  const setSidebarView = useShellStore((s) => s.setSidebarView);
  const agentOpen = useShellStore((s) => s.agentOpen);
  const toggleAgent = useShellStore((s) => s.toggleAgent);

  return (
    <aside
      className="flex flex-col items-center border-r border-[var(--app-border-subtle)] bg-[var(--app-surface-1)]"
      style={{ width: "var(--app-activity-bar-width)" }}
    >
      {/* Brand mark */}
      <Link
        to="/"
        aria-label="DB Pro"
        className="mb-2 mt-2 grid h-8 w-8 place-items-center overflow-hidden rounded-lg bg-[var(--app-surface-3)] ring-1 ring-inset ring-[var(--app-border-subtle)] transition-colors hover:bg-[var(--app-hover)] focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[var(--focus-ring-color)]"
      >
        <img
          src="/brand/db-pro-logo.svg"
          alt=""
          aria-hidden="true"
          className="h-8 w-8 object-contain"
        />
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
                    "relative flex h-10 w-10 items-center justify-center rounded-md text-[var(--app-text-muted)] transition-colors duration-100",
                    "hover:bg-[var(--app-hover)] hover:text-foreground",
                    "focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[var(--focus-ring-color)]",
                    isActive ? "bg-primary/10 text-primary" : "text-[var(--app-text-muted)]",
                  )}
                  onClick={() => setSidebarView(item.viewId)}
                  aria-label={label}
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

      {/* Bottom — agent toggle + collapse toggle */}
      <div className="flex flex-col items-center gap-0.5 pb-2">
        <Tooltip>
          <TooltipTrigger asChild>
            <button
              type="button"
              className={cn(
                "flex h-8 w-8 items-center justify-center rounded-md text-[var(--app-text-muted)] transition-colors duration-100 hover:bg-[var(--app-hover)] hover:text-foreground",
                agentOpen && "text-primary",
              )}
              onClick={toggleAgent}
              aria-label={t("shell.agentToggle")}
            >
              <Sparkles className="h-4 w-4" />
            </button>
          </TooltipTrigger>
          <TooltipContent side="right" sideOffset={8}>
            {t("shell.agentToggle")}
          </TooltipContent>
        </Tooltip>

        <Tooltip>
          <TooltipTrigger asChild>
            <button
              type="button"
              className="flex h-8 w-8 items-center justify-center rounded-md text-[var(--app-text-muted)] transition-colors duration-100 hover:bg-[var(--app-hover)] hover:text-foreground"
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
