import { Command, Search } from "lucide-react";

import { useTranslation } from "@/commons/locales/useTranslation";
import { useCommandStore } from "@/commons/stores/command.store";
import { useQuickOpenStore } from "@/commons/stores/quick-open.store";
import { isMac } from "@/commons/utils/platform";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";

export function Topbar() {
  const { t } = useTranslation();

  return (
    <header
      className="flex items-center border-b border-[var(--border-subtle)] bg-[var(--surface-panel)] px-3"
      style={{ height: "var(--app-topbar-height)" }}
      role="banner"
    >
      {/* Left — reserve space for native traffic lights only on macOS */}
      <div className={`flex items-center gap-2 ${isMac ? "pl-14" : "pl-3"}`}>
        <span className="text-[13px] font-semibold text-[var(--text-secondary)]">DB Pro</span>
      </div>

      <div className="flex-1" />

      {/* Right actions */}
      <div className="flex items-center gap-2">
        <Tooltip>
          <TooltipTrigger asChild>
            <button
              type="button"
              className="flex h-7 w-[clamp(150px,24vw,240px)] items-center gap-1.5 rounded-md border border-[var(--border-default)] bg-[var(--surface-editor)] px-2.5 text-xs text-[var(--text-secondary)] transition-colors hover:border-[var(--border-strong)] hover:text-foreground"
              onClick={() => useQuickOpenStore.getState().open()}
            >
              <Search className="h-3.5 w-3.5 shrink-0" />
              <span className="flex-1 text-left">{t("shell.topbar.search")}</span>
              <kbd className="ml-1 rounded border border-[var(--border-strong)] bg-background px-1 py-px text-[11px] font-medium text-[var(--text-tertiary)]">
                {isMac ? "⌘P" : "Ctrl+P"}
              </kbd>
            </button>
          </TooltipTrigger>
          <TooltipContent side="bottom" sideOffset={4}>
            {t("shell.topbar.quickOpen")}
          </TooltipContent>
        </Tooltip>

        <Tooltip>
          <TooltipTrigger asChild>
            <button
              type="button"
              className="grid h-7 w-7 place-items-center rounded-md text-[var(--text-secondary)] transition-colors hover:bg-[var(--surface-hover)] hover:text-foreground"
              onClick={() => useCommandStore.getState().open()}
              aria-label={t("shell.topbar.commandMenu")}
            >
              <Command className="h-4 w-4" />
            </button>
          </TooltipTrigger>
          <TooltipContent side="bottom" sideOffset={4}>
            {t("shell.topbar.commandMenu")} · {isMac ? "⌘K" : "Ctrl+K"}
          </TooltipContent>
        </Tooltip>
      </div>
    </header>
  );
}
