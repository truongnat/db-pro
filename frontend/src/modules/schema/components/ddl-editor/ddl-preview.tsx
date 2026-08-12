import { useTranslation } from "@/commons/locales/useTranslation";

interface DdlPreviewProps {
  sql: string;
}

export function DdlPreview({ sql }: DdlPreviewProps) {
  const { t } = useTranslation();

  if (!sql) {
    return (
      <div className="rounded-sm border border-[var(--border-subtle)] bg-muted p-4 text-sm italic text-[var(--text-secondary)]">
        {t("schema.ddlPreview")}
      </div>
    );
  }

  return (
    <pre
      className="overflow-auto rounded-sm border border-[var(--border-subtle)] bg-muted p-4 font-mono text-xs leading-relaxed text-foreground"
      style={{ maxHeight: "300px" }}
    >
      <code>{sql}</code>
    </pre>
  );
}
