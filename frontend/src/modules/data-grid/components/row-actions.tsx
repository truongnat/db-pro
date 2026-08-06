import { useTranslation } from "@/commons/locales/useTranslation";

interface RowActionsProps {
  onDelete: () => void;
  isDeleting: boolean;
}

export function RowActions({ onDelete, isDeleting }: RowActionsProps) {
  const { t } = useTranslation();

  return (
    <button
      className="rounded px-1.5 py-0.5 text-xs transition-colors hover:bg-[var(--color-error, #ef4444)] hover:text-white disabled:opacity-40"
      style={{ color: "var(--color-error, #ef4444)" }}
      disabled={isDeleting}
      onClick={(e) => {
        e.stopPropagation();
        if (window.confirm(t("dataGrid.confirmDelete"))) {
          onDelete();
        }
      }}
      type="button"
      title={t("common.actions.delete")}
    >
      &times;
    </button>
  );
}
