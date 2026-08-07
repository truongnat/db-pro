import { useTranslation } from "@/commons/locales/useTranslation";

interface EmptyStateProps {
  message?: string;
}

export function EmptyState({ message }: EmptyStateProps) {
  const { t } = useTranslation();
  return (
    <div className="flex h-full min-h-0 items-center justify-center py-12">
      <p className="text-sm text-[var(--app-text-muted)]">
        {message ?? t("dataGrid.noData")}
      </p>
    </div>
  );
}
