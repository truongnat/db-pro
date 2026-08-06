import { useTranslation } from "@/commons/locales/useTranslation";

interface EmptyStateProps {
  message?: string;
}

export function EmptyState({ message }: EmptyStateProps) {
  const { t } = useTranslation();
  return (
    <div className="flex flex-1 items-center justify-center py-12">
      <p className="text-sm text-muted-foreground">
        {message ?? t("dataGrid.noData")}
      </p>
    </div>
  );
}
