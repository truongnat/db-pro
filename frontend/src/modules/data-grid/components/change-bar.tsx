import { useTranslation } from "@/commons/locales/useTranslation";
import { Button } from "@/components/ui/button";
import { Trash2 } from "lucide-react";

import type { StagedChange } from "../state/staged-changes.store";

interface ChangeBarProps {
  changes: StagedChange[];
  isApplying: boolean;
  onApply: () => void;
  onRevertAll: () => void;
  onRetryFailed?: () => void;
  onBatchDelete?: (selectedRows: Set<number>) => void;
  selectedRows?: Set<number>;
}

export function ChangeBar({
  changes,
  isApplying,
  onApply,
  onRevertAll,
  onRetryFailed,
  onBatchDelete,
  selectedRows,
}: ChangeBarProps) {
  const { t } = useTranslation();

  if (changes.length === 0 && (!selectedRows || selectedRows.size === 0)) return null;

  const edits = changes.filter((c) => c.kind === "cell-edit").length;
  const deletes = changes.filter((c) => c.kind === "row-delete").length;
  const failed = changes.filter((c) => "error" in c && c.error).length;

  const parts: string[] = [];
  if (edits > 0) parts.push(t("dataGrid.changes.edits", { count: edits }));
  if (deletes > 0) parts.push(t("dataGrid.changes.deletes", { count: deletes }));

  const selectionCount = selectedRows?.size ?? 0;

  return (
    <div className="flex items-center gap-2 border-b border-[var(--app-border-subtle)] bg-background px-3 py-1.5 text-xs">
      {failed > 0 ? (
        <>
          <span className="font-medium text-destructive">
            {t("dataGrid.changes.applyPartial", {
              applied: changes.length - failed,
              total: changes.length,
            })}
          </span>
          <span className="text-[var(--app-text-muted)]">
            {t("dataGrid.changes.failedCount", { count: failed })}
          </span>
        </>
      ) : changes.length > 0 ? (
        <>
          <span className="font-medium text-foreground">
            {t("dataGrid.changes.pending", { count: changes.length })}
          </span>
          <span className="text-[var(--app-text-muted)]">{parts.join(", ")}</span>
        </>
      ) : null}
      <div className="flex-1" />
      {selectionCount > 0 && onBatchDelete && (
        <Button
          type="button"
          variant="ghost"
          size="sm"
          className="h-auto px-2 py-0.5 text-xs text-destructive hover:text-destructive"
          onClick={() => onBatchDelete(selectedRows!)}
          disabled={isApplying}
        >
          <Trash2 className="mr-1 h-3 w-3" />
          {t("dataGrid.changes.deleteSelected", { count: selectionCount })}
        </Button>
      )}
      {failed > 0 && onRetryFailed && (
        <Button
          type="button"
          variant="ghost"
          size="sm"
          className="h-auto px-2 py-0.5 text-xs text-warning hover:text-warning"
          onClick={onRetryFailed}
          disabled={isApplying}
        >
          {t("dataGrid.changes.retryFailed")}
        </Button>
      )}
      {changes.length > 0 && (
        <Button
          type="button"
          variant="ghost"
          size="sm"
          className="h-auto px-2 py-0.5 text-xs text-[var(--app-text-muted)] hover:text-foreground"
          onClick={onRevertAll}
          disabled={isApplying}
        >
          {t("dataGrid.changes.revertAll")}
        </Button>
      )}
      {changes.length > 0 && (
        <Button
          type="button"
          size="sm"
          className="h-auto px-3 py-0.5 text-xs"
          onClick={onApply}
          disabled={isApplying}
        >
          {isApplying ? t("dataGrid.changes.applying") : t("dataGrid.changes.apply")}
        </Button>
      )}
    </div>
  );
}
