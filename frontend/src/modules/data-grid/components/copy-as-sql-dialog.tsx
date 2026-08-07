import { useCallback, useMemo, useState } from "react";

import { useTranslation } from "@/commons/locales/useTranslation";
import { Button } from "@/components/ui/button";
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
      className="fixed inset-0 z-[var(--z-overlay)] flex items-center justify-center"
      style={{ backgroundColor: "rgba(0, 0, 0, 0.5)" }}
      onClick={onClose}
    >
      <div
        className="flex max-h-[80vh] w-[600px] flex-col rounded-md border border-border bg-muted shadow-lg"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between border-b border-border p-4">
          <h3 className="text-sm font-medium text-foreground">
            {t("dataGrid.copyAsSql")}
          </h3>
          <Button
            type="button"
            variant="ghost"
            size="icon-sm"
            onClick={onClose}
            className="text-lg text-muted-foreground"
          >
            ×
          </Button>
        </div>

        <div className="flex flex-col gap-3 p-4">
          <div className="flex gap-2">
            {(["insert", "update", "delete"] as SqlFormat[]).map((f) => (
              <Button
                key={f}
                type="button"
                variant="outline"
                size="sm"
                onClick={() => setFormat(f)}
                className={
                  "px-3 text-xs font-medium " +
                  (format === f
                    ? "border-primary bg-primary text-primary-foreground hover:bg-primary"
                    : "text-foreground")
                }
              >
                {f.toUpperCase()}
              </Button>
            ))}
          </div>

          <pre className="max-h-[400px] overflow-auto rounded-sm border border-border bg-background p-3 font-mono text-xs leading-relaxed text-foreground">
            <code>{generatedSql || t("dataGrid.noData")}</code>
          </pre>

          <div className="flex justify-end gap-2">
            <Button
              type="button"
              variant="outline"
              onClick={onClose}
              className="text-xs"
            >
              {t("common.actions.cancel")}
            </Button>
            <Button
              type="button"
              onClick={handleCopy}
              disabled={!generatedSql}
              className="text-xs font-medium"
            >
              {copied ? t("schema.copied") : t("dataGrid.copyToClipboard")}
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}
