import { useTranslation } from "@/commons/locales/useTranslation";

interface DdlPreviewProps {
  sql: string;
}

export function DdlPreview({ sql }: DdlPreviewProps) {
  const { t } = useTranslation();

  if (!sql) {
    return (
      <div
        className="rounded-[var(--radius-sm)] border p-4 text-sm italic"
        style={{
          borderColor: "var(--color-border)",
          color: "var(--color-text-secondary)",
          backgroundColor: "var(--color-bg-secondary, var(--color-bg))",
        }}
      >
        {t("schema.ddlPreview")}
      </div>
    );
  }

  return (
    <pre
      className="overflow-auto rounded-[var(--radius-sm)] border p-4 font-mono text-xs leading-relaxed"
      style={{
        borderColor: "var(--color-border)",
        color: "var(--color-text)",
        backgroundColor: "var(--color-bg-secondary, var(--color-bg))",
        maxHeight: "300px",
      }}
    >
      <code>{sql}</code>
    </pre>
  );
}
