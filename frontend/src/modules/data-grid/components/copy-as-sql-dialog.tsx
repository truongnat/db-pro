import { useCallback, useMemo, useState } from "react";

import { useTranslation } from "@/commons/locales/useTranslation";
import type { ColumnMeta, Row } from "@/modules/query/types/query.types";
import {
  generateDeleteSQL,
  generateInsertSQL,
  generateUpdateSQL,
} from "@/modules/data-grid/services/sql-generator";

interface CopyAsSqlDialogProps {
  open: boolean;
  onClose: () => void;
  schema: string;
  table: string;
  columns: ColumnMeta[];
  rows: Row[];
  pkColumns: string[];
}

type SqlFormat = "insert" | "update" | "delete";

export function CopyAsSqlDialog({
  open,
  onClose,
  schema,
  table,
  columns,
  rows,
  pkColumns,
}: CopyAsSqlDialogProps) {
  const { t } = useTranslation();
  const [format, setFormat] = useState<SqlFormat>("insert");
  const [copied, setCopied] = useState(false);

  const generatedSql = useMemo(() => {
    if (rows.length === 0) return "";

    switch (format) {
      case "insert":
        return generateInsertSQL(schema, table, columns, rows);
      case "update":
        return rows
          .map((row) => generateUpdateSQL(schema, table, columns, row, pkColumns))
          .filter(Boolean)
          .join("\n\n");
      case "delete":
        return rows
          .map((row) => generateDeleteSQL(schema, table, columns, row, pkColumns))
          .filter(Boolean)
          .join("\n\n");
      default:
        return "";
    }
  }, [format, schema, table, columns, rows, pkColumns]);

  const handleCopy = useCallback(async () => {
    if (!generatedSql) return;
    await navigator.clipboard.writeText(generatedSql);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }, [generatedSql]);

  if (!open) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center"
      style={{ backgroundColor: "rgba(0, 0, 0, 0.5)" }}
      onClick={onClose}
    >
      <div
        className="flex max-h-[80vh] w-[600px] flex-col rounded-[var(--radius-md)] border shadow-lg"
        style={{
          borderColor: "var(--color-border)",
          backgroundColor: "var(--color-bg-secondary, var(--color-bg))",
        }}
        onClick={(e) => e.stopPropagation()}
      >
        <div
          className="flex items-center justify-between border-b p-4"
          style={{ borderColor: "var(--color-border)" }}
        >
          <h3 className="text-sm font-medium" style={{ color: "var(--color-text)" }}>
            {t("dataGrid.copyAsSql")}
          </h3>
          <button
            type="button"
            onClick={onClose}
            className="text-lg"
            style={{ color: "var(--color-text-secondary)" }}
          >
            ×
          </button>
        </div>

        <div className="flex flex-col gap-3 p-4">
          <div className="flex gap-2">
            {(["insert", "update", "delete"] as SqlFormat[]).map((f) => (
              <button
                key={f}
                type="button"
                onClick={() => setFormat(f)}
                className="rounded-[var(--radius-sm)] border px-3 py-1.5 text-xs font-medium transition-colors"
                style={{
                  borderColor: format === f ? "var(--color-primary, #3b82f6)" : "var(--color-border)",
                  backgroundColor: format === f ? "var(--color-primary, #3b82f6)" : "transparent",
                  color: format === f ? "white" : "var(--color-text)",
                }}
              >
                {f.toUpperCase()}
              </button>
            ))}
          </div>

          <pre
            className="max-h-[400px] overflow-auto rounded-[var(--radius-sm)] border p-3 font-mono text-xs leading-relaxed"
            style={{
              borderColor: "var(--color-border)",
              color: "var(--color-text)",
              backgroundColor: "var(--color-bg)",
            }}
          >
            <code>{generatedSql || t("dataGrid.noData")}</code>
          </pre>

          <div className="flex justify-end gap-2">
            <button
              type="button"
              onClick={onClose}
              className="rounded-[var(--radius-sm)] border px-3 py-1.5 text-xs transition-colors hover:bg-[var(--color-bg)]"
              style={{ borderColor: "var(--color-border)", color: "var(--color-text)" }}
            >
              {t("common.actions.cancel")}
            </button>
            <button
              type="button"
              onClick={handleCopy}
              disabled={!generatedSql}
              className="rounded-[var(--radius-sm)] px-3 py-1.5 text-xs font-medium text-white transition-colors disabled:opacity-50"
              style={{ backgroundColor: "var(--color-primary, #3b82f6)" }}
            >
              {copied ? t("schema.copied") : t("dataGrid.copyToClipboard")}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
