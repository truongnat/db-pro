import { useTranslation } from "@/commons/locales/useTranslation";
import { Button } from "@/components/ui/button";
import type { QueryTiming } from "@/commons/types/workspace.types";
import { useQueryTimer } from "../hooks/use-query-timer";

/** Threshold above which we show a large-result warning. */
const LARGE_RESULT_THRESHOLD = 10_000;

function formatElapsed(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  return `${(ms / 1000).toFixed(1)}s`;
}

function formatTiming(t: QueryTiming): string {
  const parts: string[] = [`${t.totalMs}ms`];
  if (t.serverMs > 0 || t.fetchMs > 0) {
    const detail: string[] = [];
    if (t.serverMs > 0) detail.push(`server: ${t.serverMs}ms`);
    if (t.fetchMs > 0) detail.push(`fetch: ${t.fetchMs}ms`);
    parts.push(`(${detail.join(", ")})`);
  }
  return parts.join(" ");
}

interface QueryStatusBarProps {
  tabId: string;
  status: "idle" | "running" | "success" | "error" | "cancelled";
  executionStartedAt: number | null;
  rowCount: number;
  timing: QueryTiming | null;
  onCancel: () => void;
}

export function QueryStatusBar({
  status,
  executionStartedAt,
  rowCount,
  timing,
  onCancel,
}: QueryStatusBarProps) {
  const { t } = useTranslation();
  const elapsed = useQueryTimer(status, executionStartedAt);

  // Running state
  if (status === "running") {
    return (
      <div className="flex items-center gap-2 border-t border-[var(--app-border-subtle)] bg-muted/30 px-3 text-[11px]">
        <span className="inline-block h-2 w-2 animate-pulse rounded-full bg-[var(--app-primary)]" />
        <span className="text-[var(--app-text-muted)]">
          {t("query.statusRunning")} · {formatElapsed(elapsed)}
        </span>
        <div className="flex-1" />
        <Button
          type="button"
          variant="ghost"
          size="sm"
          className="h-5 px-2 text-[11px]"
          onClick={onCancel}
        >
          {t("common.actions.cancel")}
        </Button>
      </div>
    );
  }

  // Success state with timing
  if (status === "success" && timing) {
    const isLarge = rowCount >= LARGE_RESULT_THRESHOLD;
    return (
      <div className="flex items-center gap-2 border-t border-[var(--app-border-subtle)] bg-muted/30 px-3 text-[11px]">
        {isLarge && (
          <span className="rounded bg-[var(--app-warning)]/15 px-1.5 py-0.5 text-[var(--app-warning)]">
            {t("query.largeResultWarning")}
          </span>
        )}
        <span className="text-[var(--app-text-muted)]">
          {t("query.rowsAffected", { count: rowCount })} · {formatTiming(timing)}
        </span>
      </div>
    );
  }

  // Error state
  if (status === "error") {
    return (
      <div className="flex items-center gap-2 border-t border-[var(--app-border-subtle)] bg-destructive/10 px-3 text-[11px] text-destructive">
        <span>{t("query.statusError")}</span>
      </div>
    );
  }

  // Cancelled state
  if (status === "cancelled") {
    return (
      <div className="flex items-center gap-2 border-t border-[var(--app-border-subtle)] bg-muted/30 px-3 text-[11px]">
        <span className="text-[var(--app-text-muted)]">{t("query.statusCancelled")}</span>
      </div>
    );
  }

  // Idle — nothing to show
  return null;
}
