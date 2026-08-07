import { useTranslation } from "@/commons/locales/useTranslation";
import { Button } from "@/components/ui/button";

import type { StagedChange } from "../state/staged-changes.store";

interface ChangeBarProps {
  changes: StagedChange[];
  isApplying: boolean;
  onApply: () => void;
  onRevertAll: () => void;
}

export function ChangeBar({ changes, isApplying, onApply, onRevertAll }: ChangeBarProps) {
  const { t } = useTranslation();

  if (changes.length === 0) return null;

  const edits = changes.filter((c) => c.kind === "cell-edit").length;
  const deletes = changes.filter((c) => c.kind === "row-delete").length;

  const parts: string[] = [];
  if (edits > 0) parts.push(t("dataGrid.changes.edits", { count: edits }));
  if (deletes > 0) parts.push(t("dataGrid.changes.deletes", { count: deletes }));

  return (
    <div className="flex items-center gap-2 border-b border-border bg-card px-3 py-1.5 text-xs">
      <span className="font-medium text-foreground">
        {t("dataGrid.changes.pending", { count: changes.length })}
      </span>
      <span className="text-muted-foreground">
        {parts.join(", ")}
      </span>
      <div className="flex-1" />
      <Button
        type="button"
        variant="ghost"
        size="sm"
        className="h-auto px-2 py-0.5 text-xs text-muted-foreground hover:text-foreground"
        onClick={onRevertAll}
        disabled={isApplying}
      >
        {t("dataGrid.changes.revertAll")}
      </Button>
      <Button
        type="button"
        size="sm"
        className="h-auto px-3 py-0.5 text-xs"
        onClick={onApply}
        disabled={isApplying}
      >
        {isApplying ? t("dataGrid.changes.applying") : t("dataGrid.changes.apply")}
      </Button>
    </div>
  );
}
