import { useCallback, useState } from "react";

import { useTranslation } from "@/commons/locales/useTranslation";

import { useExecuteDdl } from "../queries/schema.queries";
import type { SchemaColumnDto, SchemaIndexDto } from "../types/schema.types";

interface IndexManagerProps {
  connectionId: string;
  schema: string;
  table: string;
  columns: SchemaColumnDto[];
  indexes: SchemaIndexDto[];
}

export function IndexManager({
  connectionId,
  schema,
  table,
  columns,
  indexes,
}: IndexManagerProps) {
  const { t } = useTranslation();
  const executeDdl = useExecuteDdl(connectionId);

  const [indexName, setIndexName] = useState("");
  const [selectedColumns, setSelectedColumns] = useState<string[]>([]);
  const [unique, setUnique] = useState(false);

  const toggleColumn = useCallback((col: string) => {
    setSelectedColumns((prev) =>
      prev.includes(col) ? prev.filter((c) => c !== col) : [...prev, col],
    );
  }, []);

  const handleCreate = useCallback(() => {
    if (!indexName.trim() || selectedColumns.length === 0) return;
    const cols = selectedColumns.map((c) => `"${c}"`).join(", ");
    const uniqueKw = unique ? "UNIQUE " : "";
    const sql = `CREATE ${uniqueKw}INDEX "${indexName.trim()}" ON "${schema}"."${table}" (${cols})`;
    executeDdl.mutate(sql, {
      onSuccess: () => {
        setIndexName("");
        setSelectedColumns([]);
        setUnique(false);
      },
    });
  }, [indexName, selectedColumns, unique, schema, table, executeDdl]);

  const handleDrop = useCallback(
    (idxName: string) => {
      const sql = `DROP INDEX "${schema}"."${idxName}"`;
      executeDdl.mutate(sql);
    },
    [schema, executeDdl],
  );

  return (
    <div className="flex flex-col gap-4 p-3">
      <div>
        <h4 className="mb-2 text-xs font-semibold" style={{ color: "var(--color-text)" }}>
          {t("schema.existingIndexes")}
        </h4>
        {indexes.length === 0 ? (
          <p className="text-xs italic" style={{ color: "var(--color-text-secondary)" }}>
            {t("schema.noIndexes")}
          </p>
        ) : (
          <div className="space-y-1">
            {indexes.map((idx) => (
              <div
                key={idx.name}
                className="group flex items-center justify-between rounded-[var(--radius-sm)] px-2 py-1 text-xs hover:bg-[var(--color-bg)]"
              >
                <span className="font-mono" style={{ color: "var(--color-text)" }}>
                  {idx.name} ({idx.columns.join(", ")}){idx.unique ? " UNIQUE" : ""}
                </span>
                <button
                  type="button"
                  className="opacity-0 transition-opacity group-hover:opacity-100"
                  style={{ color: "var(--color-error, #ef4444)" }}
                  onClick={() => handleDrop(idx.name)}
                >
                  {t("common.actions.delete")}
                </button>
              </div>
            ))}
          </div>
        )}
      </div>

      <div
        className="rounded-[var(--radius-sm)] border p-3"
        style={{ borderColor: "var(--color-border)" }}
      >
        <h4 className="mb-2 text-xs font-semibold" style={{ color: "var(--color-text)" }}>
          {t("schema.createIndex")}
        </h4>

        <div className="space-y-2">
          <div>
            <label className="mb-1 block text-xs" style={{ color: "var(--color-text-secondary)" }}>
              {t("schema.ddlIndexName")}
            </label>
            <input
              type="text"
              value={indexName}
              onChange={(e) => setIndexName(e.target.value)}
              className="w-full rounded-[var(--radius-sm)] border px-2 py-1 text-xs outline-none focus:border-[var(--color-primary,#3b82f6)]"
              style={{
                borderColor: "var(--color-border)",
                backgroundColor: "var(--color-bg)",
                color: "var(--color-text)",
              }}
            />
          </div>

          <div>
            <label className="mb-1 block text-xs" style={{ color: "var(--color-text-secondary)" }}>
              {t("schema.ddlIndexColumns")}
            </label>
            <div className="flex flex-wrap gap-1">
              {columns.map((col) => (
                <button
                  key={col.name}
                  type="button"
                  className="rounded-[var(--radius-sm)] border px-2 py-0.5 text-xs transition-colors"
                  style={{
                    borderColor: selectedColumns.includes(col.name)
                      ? "var(--color-primary, #3b82f6)"
                      : "var(--color-border)",
                    backgroundColor: selectedColumns.includes(col.name)
                      ? "var(--color-primary, #3b82f6)"
                      : "var(--color-bg)",
                    color: selectedColumns.includes(col.name)
                      ? "white"
                      : "var(--color-text)",
                  }}
                  onClick={() => toggleColumn(col.name)}
                >
                  {col.name}
                </button>
              ))}
            </div>
          </div>

          <label className="flex items-center gap-2 text-xs" style={{ color: "var(--color-text)" }}>
            <input
              type="checkbox"
              checked={unique}
              onChange={(e) => setUnique(e.target.checked)}
            />
            {t("schema.ddlUnique")}
          </label>

          <button
            type="button"
            disabled={!indexName.trim() || selectedColumns.length === 0 || executeDdl.isPending}
            className="rounded-[var(--radius-sm)] px-3 py-1.5 text-xs text-white disabled:opacity-50"
            style={{ backgroundColor: "var(--color-primary, #3b82f6)" }}
            onClick={handleCreate}
          >
            {t("schema.createIndex")}
          </button>
        </div>
      </div>
    </div>
  );
}
