import { useCallback, useState } from "react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Checkbox } from "@/components/ui/checkbox";
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

export function IndexManager({ connectionId, schema, table, columns, indexes }: IndexManagerProps) {
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
        <h4 className="mb-2 text-xs font-semibold text-foreground">
          {t("schema.existingIndexes")}
        </h4>
        {indexes.length === 0 ? (
          <p className="text-xs italic text-[var(--app-text-muted)]">{t("schema.noIndexes")}</p>
        ) : (
          <div className="space-y-1">
            {indexes.map((idx) => (
              <div
                key={idx.name}
                className="group flex items-center justify-between rounded-sm px-2 py-1 text-xs hover:bg-background"
              >
                <span className="font-mono text-foreground">
                  {idx.name} ({idx.columns.join(", ")}){idx.unique ? " UNIQUE" : ""}
                </span>
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  className="text-destructive opacity-0 transition-opacity group-hover:opacity-100"
                  onClick={() => handleDrop(idx.name)}
                >
                  {t("common.actions.delete")}
                </Button>
              </div>
            ))}
          </div>
        )}
      </div>

      <div className="rounded-sm border border-[var(--app-border-subtle)] p-3">
        <h4 className="mb-2 text-xs font-semibold text-foreground">{t("schema.createIndex")}</h4>

        <div className="space-y-2">
          <div>
            <Label htmlFor="index-name" className="mb-1 block text-xs text-[var(--app-text-muted)]">
              {t("schema.ddlIndexName")}
            </Label>
            <Input
              id="index-name"
              type="text"
              value={indexName}
              onChange={(e) => setIndexName(e.target.value)}
              className="w-full rounded-sm border border-[var(--app-border)] bg-background px-2 py-1 text-xs text-foreground outline-none focus:border-primary"
            />
          </div>

          <div>
            <Label className="mb-1 block text-xs text-[var(--app-text-muted)]">
              {t("schema.ddlIndexColumns")}
            </Label>
            <div className="flex flex-wrap gap-1">
              {columns.map((col) => (
                <Button
                  key={col.name}
                  type="button"
                  size="sm"
                  variant={selectedColumns.includes(col.name) ? "default" : "outline"}
                  onClick={() => toggleColumn(col.name)}
                >
                  {col.name}
                </Button>
              ))}
            </div>
          </div>

          <Label className="flex items-center gap-2 text-xs text-foreground">
            <Checkbox checked={unique} onCheckedChange={(checked) => setUnique(checked === true)} />
            {t("schema.ddlUnique")}
          </Label>

          <Button
            type="button"
            size="sm"
            disabled={!indexName.trim() || selectedColumns.length === 0 || executeDdl.isPending}
            onClick={handleCreate}
          >
            {t("schema.createIndex")}
          </Button>
        </div>
      </div>
    </div>
  );
}
