import { useTranslation } from "@/commons/locales/useTranslation";
import { Button } from "@/components/ui/button";

interface RowActionsProps {
  onDelete: () => void;
  isDeleting: boolean;
}

export function RowActions({ onDelete, isDeleting }: RowActionsProps) {
  const { t } = useTranslation();

  return (
    <Button
      type="button"
      variant="ghost"
      size="sm"
      className="h-auto px-1.5 py-0.5 text-xs text-destructive hover:bg-destructive hover:text-white disabled:opacity-40"
      disabled={isDeleting}
      onClick={(e) => {
        e.stopPropagation();
        if (window.confirm(t("dataGrid.confirmDelete"))) {
          onDelete();
        }
      }}
      title={t("common.actions.delete")}
    >
      &times;
    </Button>
  );
}
