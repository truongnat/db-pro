import { useState } from "react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
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
  const {
    data: diff,
    isLoading,
    error,
  } = useDiffTableData(sourceId, targetId, schema || null, table || null, enabled);

  if (!sourceId || !targetId) {
    return (
      <div className="flex items-center justify-center py-12">
        <p className="text-[var(--text-secondary)]">{t("schema.crossConn.selectTwo")}</p>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-4">
      <div className="grid grid-cols-3 gap-3">
        <Input
          value={schema}
          onChange={(e) => setSchema(e.target.value)}
          placeholder={t("schema.crossConn.schemaName")}
          className="rounded-sm border border-[var(--border-default)] px-3 py-2 text-sm text-foreground"
        />
        <Input
          value={table}
          onChange={(e) => setTable(e.target.value)}
          placeholder={t("schema.crossConn.tableName")}
          className="rounded-sm border border-[var(--border-default)] px-3 py-2 text-sm text-foreground"
        />
        <Button
          type="button"
          onClick={() => setEnabled(true)}
          disabled={isLoading || !schema || !table}
        >
          {isLoading ? t("common.states.loading") : t("schema.crossConn.compare")}
        </Button>
      </div>

      {error && (
        <div className="rounded-sm bg-destructive px-3 py-2 text-sm text-white">
          {(error as Error).message}
        </div>
      )}

      {diff && (
        <div className="rounded-sm border border-[var(--border-subtle)] p-4">
          <h4 className="mb-3 text-sm font-medium text-foreground">
            {diff.schema}.{diff.table}
          </h4>
          <div className="grid grid-cols-3 gap-4 text-sm">
            <div>
              <p className="text-[var(--text-secondary)]">{t("schema.crossConn.sourceCount")}</p>
              <p className="text-lg font-medium text-foreground">{diff.sourceRowCount}</p>
            </div>
            <div>
              <p className="text-[var(--text-secondary)]">{t("schema.crossConn.targetCount")}</p>
              <p className="text-lg font-medium text-foreground">{diff.targetRowCount}</p>
            </div>
            <div>
              <p className="text-[var(--text-secondary)]">{t("schema.crossConn.difference")}</p>
              <p
                className={`text-lg font-medium ${
                  diff.rowCountDiff === 0 ? "text-success" : "text-destructive"
                }`}
              >
                {diff.rowCountDiff > 0 ? "+" : ""}
                {diff.rowCountDiff}
              </p>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
