import { useTranslation } from "@/commons/locales/useTranslation";

interface ExportProgressProps {
  format: string;
  rowCount: number | null;
}

export function ExportProgress({ format, rowCount }: ExportProgressProps) {
  const { t } = useTranslation();

  return (
    <div className="flex items-center gap-2 px-3 py-2 text-sm text-[var(--app-text-muted)]">
      <span className="inline-block h-3 w-3 animate-spin rounded-full border-2 border-current border-t-transparent" />
      <span>
        {t("export.exporting")} ({format.toUpperCase()})
        {rowCount !== null && ` — ${rowCount} rows`}
      </span>
    </div>
  );
}
