import { useCallback, useMemo, useState } from "react";

import { Button } from "@/components/ui/button";
import { useTranslation } from "@/commons/locales/useTranslation";

import { useExecuteDdl } from "../../queries/schema.queries";
import {
  generateDdlPreview,
  type ColumnDef,
  type DdlOperation,
} from "../../services/ddl-builder";

import { ColumnDefRow } from "./column-def-row";
import { DdlPreview } from "./ddl-preview";
import { DdlTypeSelector } from "./ddl-type-selector";

interface DdlEditorProps {
  connectionId: string;
  schema: string;
  table: string;
}

const EMPTY_COLUMN: ColumnDef = {
  name: "",
  dataType: "TEXT",
  nullable: true,
  defaultValue: "",
  isPk: false,
};

const NEEDS_COLUMNS: DdlOperation[] = ["createTable", "addColumn"];
const NEEDS_COLUMN_NAME: DdlOperation[] = ["dropColumn"];
const NEEDS_NEW_NAME: DdlOperation[] = ["renameTable"];
const NEEDS_SELECT_SQL: DdlOperation[] = ["createView"];
const NEEDS_INDEX_CONFIG: DdlOperation[] = ["createIndex"];
const NEEDS_INDEX_NAME: DdlOperation[] = ["dropIndex"];

export function DdlEditor({ connectionId, schema, table }: DdlEditorProps) {
  const { t } = useTranslation();
  const executeDdl = useExecuteDdl(connectionId);

  const [operation, setOperation] = useState<DdlOperation>("createTable");
  const [columns, setColumns] = useState<ColumnDef[]>([{ ...EMPTY_COLUMN, name: "id", isPk: true, nullable: false }]);
  const [columnName, setColumnName] = useState("");
  const [newName, setNewName] = useState("");
  const [selectSql, setSelectSql] = useState("");
  const [indexName, setIndexName] = useState("");
  const [indexColumns, setIndexColumns] = useState("");
  const [unique, setUnique] = useState(false);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  const extra = useMemo(
    () => ({
      columnName,
      newName,
      selectSql,
      indexName,
      indexColumns,
      unique: String(unique),
    }),
    [columnName, newName, selectSql, indexName, indexColumns, unique],
  );

  const previewSql = useMemo(
    () => generateDdlPreview(operation, schema, table, columns, extra),
    [operation, schema, table, columns, extra],
  );

  const handleColumnChange = useCallback(
    (index: number, field: keyof ColumnDef, value: string | boolean) => {
      setColumns((prev) =>
        prev.map((col, i) => (i === index ? { ...col, [field]: value } : col)),
      );
    },
    [],
  );

  const handleColumnRemove = useCallback((index: number) => {
    setColumns((prev) => prev.filter((_, i) => i !== index));
  }, []);

  const handleAddColumn = useCallback(() => {
    setColumns((prev) => [...prev, { ...EMPTY_COLUMN }]);
  }, []);

  const handleOperationChange = useCallback((op: DdlOperation) => {
    setOperation(op);
    setSuccessMessage(null);
  }, []);

  const handleExecute = useCallback(async () => {
    if (!previewSql) return;
    setSuccessMessage(null);
    try {
      const result = await executeDdl.mutateAsync(previewSql);
      setSuccessMessage(t("schema.ddlExecutedSuccessfully", { affected: result.affectedRows }));
    } catch {
      // error handled by mutation
    }
  }, [previewSql, executeDdl, t]);

  const showColumnDefs = NEEDS_COLUMNS.includes(operation);
  const showColumnName = NEEDS_COLUMN_NAME.includes(operation);
  const showNewName = NEEDS_NEW_NAME.includes(operation);
  const showSelectSql = NEEDS_SELECT_SQL.includes(operation);
  const showIndexConfig = NEEDS_INDEX_CONFIG.includes(operation);
  const showIndexName = NEEDS_INDEX_NAME.includes(operation);

  return (
    <div className="flex flex-col gap-4 overflow-auto p-4">
      <DdlTypeSelector operation={operation} onChange={handleOperationChange} />

      <div className="grid grid-cols-2 gap-4">
        <div className="flex flex-col gap-1">
          <label className="text-sm font-medium text-foreground">
            {t("schema.ddlSchema")}
          </label>
          <input
            type="text"
            value={schema}
            readOnly
            className="rounded-sm border border-border bg-background px-3 py-2 text-sm text-foreground opacity-60"
          />
        </div>
        <div className="flex flex-col gap-1">
          <label className="text-sm font-medium text-foreground">
            {t("schema.ddlTable")}
          </label>
          <input
            type="text"
            value={table}
            readOnly
            className="rounded-sm border border-border bg-background px-3 py-2 text-sm text-foreground opacity-60"
          />
        </div>
      </div>

      {showColumnDefs && (
        <div className="flex flex-col gap-2">
          <label className="text-sm font-medium text-foreground">
            {t("schema.ddlColumns")}
          </label>
          {columns.map((col, i) => (
            <ColumnDefRow
              key={i}
              column={col}
              index={i}
              onChange={handleColumnChange}
              onRemove={handleColumnRemove}
              canRemove={columns.length > 1}
            />
          ))}
          <Button
            type="button"
            variant="outline"
            size="sm"
            className="self-start"
            onClick={handleAddColumn}
          >
            + {t("schema.ddlAddColumn")}
          </Button>
        </div>
      )}

      {showColumnName && (
        <div className="flex flex-col gap-1">
          <label className="text-sm font-medium text-foreground">
            {t("schema.columnName")}
          </label>
          <input
            type="text"
            value={columnName}
            onChange={(e) => setColumnName(e.target.value)}
            placeholder="column_name"
            className="rounded-sm border border-border bg-background px-3 py-2 text-sm text-foreground outline-none focus:border-primary"
          />
        </div>
      )}

      {showNewName && (
        <div className="flex flex-col gap-1">
          <label className="text-sm font-medium text-foreground">
            {t("schema.ddlNewName")}
          </label>
          <input
            type="text"
            value={newName}
            onChange={(e) => setNewName(e.target.value)}
            placeholder="new_table_name"
            className="rounded-sm border border-border bg-background px-3 py-2 text-sm text-foreground outline-none focus:border-primary"
          />
        </div>
      )}

      {showSelectSql && (
        <div className="flex flex-col gap-1">
          <label className="text-sm font-medium text-foreground">
            SELECT Statement
          </label>
          <textarea
            value={selectSql}
            onChange={(e) => setSelectSql(e.target.value)}
            placeholder="SELECT * FROM ..."
            rows={4}
            className="rounded-sm border border-border px-3 py-2 font-mono text-sm text-foreground outline-none focus:border-primary"
          />
        </div>
      )}

      {showIndexName && (
        <div className="flex flex-col gap-1">
          <label className="text-sm font-medium text-foreground">
            {t("schema.ddlIndexName")}
          </label>
          <input
            type="text"
            value={indexName}
            onChange={(e) => setIndexName(e.target.value)}
            placeholder="index_name"
            className="rounded-sm border border-border bg-background px-3 py-2 text-sm text-foreground outline-none focus:border-primary"
          />
        </div>
      )}

      {showIndexConfig && (
        <>
          <div className="flex flex-col gap-1">
            <label className="text-sm font-medium text-foreground">
              {t("schema.ddlIndexName")}
            </label>
            <input
              type="text"
              value={indexName}
              onChange={(e) => setIndexName(e.target.value)}
              placeholder="idx_name"
              className="rounded-sm border border-border bg-background px-3 py-2 text-sm text-foreground outline-none focus:border-primary"
            />
          </div>
          <div className="flex flex-col gap-1">
            <label className="text-sm font-medium text-foreground">
              {t("schema.ddlIndexColumns")}
            </label>
            <input
              type="text"
              value={indexColumns}
              onChange={(e) => setIndexColumns(e.target.value)}
              placeholder="col1, col2"
              className="rounded-sm border border-border bg-background px-3 py-2 text-sm text-foreground outline-none focus:border-primary"
            />
          </div>
          <label className="flex items-center gap-2 text-sm text-foreground">
            <input
              type="checkbox"
              checked={unique}
              onChange={(e) => setUnique(e.target.checked)}
              className="h-4 w-4 rounded border border-border accent-primary"
            />
            {t("schema.ddlUnique")}
          </label>
        </>
      )}

      <div className="flex flex-col gap-2">
        <label className="text-sm font-medium text-foreground">
          {t("schema.ddlPreview")}
        </label>
        <DdlPreview sql={previewSql} />
      </div>

      <div className="flex items-center gap-3">
        <Button
          type="button"
          onClick={handleExecute}
          disabled={!previewSql || executeDdl.isPending}
        >
          {executeDdl.isPending ? t("common.states.loading") : t("schema.ddlExecute")}
        </Button>

        {executeDdl.isError && (
          <span className="text-sm text-destructive">
            {(executeDdl.error as { userMessage?: string })?.userMessage ?? t("common.states.error")}
          </span>
        )}

        {successMessage && (
          <span className="text-sm text-success">
            {successMessage}
          </span>
        )}
      </div>
    </div>
  );
}
