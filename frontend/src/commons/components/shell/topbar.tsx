import { Search } from "lucide-react";

import { useTranslation } from "@/commons/locales/useTranslation";
import { useCommandStore } from "@/commons/stores/command.store";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";

export function Topbar() {
  const { t } = useTranslation();

  return (
    <header
      className="flex items-center border-b border-[var(--app-border-subtle)] bg-[var(--app-surface-2)] px-3"
      style={{ height: "var(--app-topbar-height)" }}
      role="banner"
    >
      {/* Left spacer for Mac traffic lights area */}
      <div className="w-14" />

      <div className="flex-1" />

      {/* Right actions */}
      <div className="flex items-center gap-2">
        <Tooltip>
          <TooltipTrigger asChild>
            <button
              type="button"
              className="flex h-7 items-center gap-1.5 rounded-md border border-[var(--app-border)] bg-muted/50 px-2.5 text-xs text-[var(--app-text-muted)] transition-colors hover:border-[var(--app-border-strong)] hover:text-foreground"
              style={{ width: 200 }}
              onClick={() => useCommandStore.getState().open()}
            >
              <Search className="h-3.5 w-3.5 shrink-0" />
              <span className="flex-1 text-left">{t("shell.topbar.search")}</span>
              <kbd className="ml-1 rounded border border-[var(--app-border-strong)] bg-background px-1 py-px text-[11px] font-medium text-[var(--app-text-dim)]">
                ⌘K
              </kbd>
            </button>
          </TooltipTrigger>
          <TooltipContent side="bottom" sideOffset={4}>
            {t("shell.topbar.commandMenu")}
          </TooltipContent>
        </Tooltip>

        {/* Profile avatar */}
        <div className="grid h-6 w-6 place-items-center rounded-full bg-primary/15 text-[11px] font-semibold text-primary">
          T
        </div>
      </div>
    </header>
  );
}
