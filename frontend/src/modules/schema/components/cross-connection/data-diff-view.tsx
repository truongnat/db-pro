import { useState } from "react";

import { useTranslation } from "@/commons/locales/useTranslation";

import { useDiffTableData } from "../../queries/schema.queries";

interface DataDiffViewProps {
  sourceId: string | null;
  targetId: string | null;
}

export function DataDiffView({ sourceId, targetId }: DataDiffViewProps) {
  const { t } = useTranslation();
  const [schema, setSchema] = useState("");
  const [table, setTable] = useState("");
  const [enabled, setEnabled] = useState(false);
  const { data: diff, isLoading, error } = useDiffTableData(sourceId, targetId, schema || null, table || null, enabled);

  if (!sourceId || !targetId) {
    return (
      <div className="flex items-center justify-center py-12">
        <p style={{ color: "var(--color-text-secondary)" }}>
          {t("schema.crossConn.selectTwo")}
        </p>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-4">
      <div className="grid grid-cols-3 gap-3">
        <input
          type="text"
          value={schema}
          onChange={(e) => setSchema(e.target.value)}
          placeholder={t("schema.crossConn.schemaName")}
          className="rounded-[var(--radius-sm)] border px-3 py-2 text-sm"
          style={{ borderColor: "var(--color-border)", color: "var(--color-text)" }}
        />
        <input
          type="text"
          value={table}
          onChange={(e) => setTable(e.target.value)}
          placeholder={t("schema.crossConn.tableName")}
          className="rounded-[var(--radius-sm)] border px-3 py-2 text-sm"
          style={{ borderColor: "var(--color-border)", color: "var(--color-text)" }}
        />
        <button
          className="rounded-[var(--radius-sm)] px-3 py-2 text-sm text-white"
          style={{ backgroundColor: "var(--color-primary,#3b82f6)" }}
          onClick={() => setEnabled(true)}
          disabled={isLoading || !schema || !table}
        >
          {isLoading ? t("common.states.loading") : t("schema.crossConn.compare")}
        </button>
      </div>

      {error && (
        <div className="rounded-[var(--radius-sm)] px-3 py-2 text-sm" style={{ backgroundColor: "var(--color-error,#ef4444)", color: "white" }}>
          {(error as Error).message}
        </div>
      )}

      {diff && (
        <div className="rounded-[var(--radius-sm)] border p-4" style={{ borderColor: "var(--color-border)" }}>
          <h4 className="mb-3 text-sm font-medium" style={{ color: "var(--color-text)" }}>
            {diff.schema}.{diff.table}
          </h4>
          <div className="grid grid-cols-3 gap-4 text-sm">
            <div>
              <p style={{ color: "var(--color-text-secondary)" }}>{t("schema.crossConn.sourceCount")}</p>
              <p className="text-lg font-medium" style={{ color: "var(--color-text)" }}>{diff.sourceRowCount}</p>
            </div>
            <div>
              <p style={{ color: "var(--color-text-secondary)" }}>{t("schema.crossConn.targetCount")}</p>
              <p className="text-lg font-medium" style={{ color: "var(--color-text)" }}>{diff.targetRowCount}</p>
            </div>
            <div>
              <p style={{ color: "var(--color-text-secondary)" }}>{t("schema.crossConn.difference")}</p>
              <p
                className="text-lg font-medium"
                style={{
                  color:
                    diff.rowCountDiff === 0
                      ? "var(--color-success,#22c55e)"
                      : "var(--color-error,#ef4444)",
                }}
              >
                {diff.rowCountDiff > 0 ? "+" : ""}{diff.rowCountDiff}
              </p>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
