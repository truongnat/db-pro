import { Search } from "lucide-react";

import { useTranslation } from "@/commons/locales/useTranslation";
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
      {/* Left — Mac traffic lights spacer + branding */}
      <div className="flex items-center gap-2 pl-14">
        <span className="text-[13px] font-semibold text-[var(--text-secondary)]">DB Pro</span>
      </div>

      <div className="flex-1" />

      {/* Right actions */}
      <div className="flex items-center gap-2">
        <Tooltip>
          <TooltipTrigger asChild>
            <button
              type="button"
              className="flex h-7 items-center gap-1.5 rounded-md border border-[var(--border-default)] bg-muted/50 px-2.5 text-xs text-[var(--text-secondary)] transition-colors hover:border-[var(--border-strong)] hover:text-foreground"
              style={{ width: 200 }}
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

        {/* Profile avatar */}
        <div className="grid h-6 w-6 place-items-center rounded-full bg-primary/15 text-[11px] font-semibold text-primary">
          T
        </div>
      </div>
    </header>
  );
}
