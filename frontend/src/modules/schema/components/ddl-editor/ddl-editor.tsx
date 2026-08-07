import { useCallback, useMemo, useState } from "react";

import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
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
          <Label htmlFor="ddl-schema" className="text-sm font-medium text-foreground">
            {t("schema.ddlSchema")}
          </Label>
          <Input
            id="ddl-schema"
            value={schema}
            readOnly
            className="opacity-60"
          />
        </div>
        <div className="flex flex-col gap-1">
          <Label htmlFor="ddl-table" className="text-sm font-medium text-foreground">
            {t("schema.ddlTable")}
          </Label>
          <Input
            id="ddl-table"
            value={table}
            readOnly
            className="opacity-60"
          />
        </div>
      </div>

      {showColumnDefs && (
        <div className="flex flex-col gap-2">
          <Label className="text-sm font-medium text-foreground">
            {t("schema.ddlColumns")}
          </Label>
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
          <Label htmlFor="ddl-column-name" className="text-sm font-medium text-foreground">
            {t("schema.columnName")}
          </Label>
          <Input
            id="ddl-column-name"
            value={columnName}
            onChange={(e) => setColumnName(e.target.value)}
            placeholder="column_name"
          />
        </div>
      )}

      {showNewName && (
        <div className="flex flex-col gap-1">
          <Label htmlFor="ddl-new-name" className="text-sm font-medium text-foreground">
            {t("schema.ddlNewName")}
          </Label>
          <Input
            id="ddl-new-name"
            value={newName}
            onChange={(e) => setNewName(e.target.value)}
            placeholder="new_table_name"
          />
        </div>
      )}

      {showSelectSql && (
        <div className="flex flex-col gap-1">
          <Label htmlFor="ddl-select-sql" className="text-sm font-medium text-foreground">
            SELECT Statement
          </Label>
          <Textarea
            id="ddl-select-sql"
            value={selectSql}
            onChange={(e) => setSelectSql(e.target.value)}
            placeholder="SELECT * FROM ..."
            rows={4}
            className="font-mono"
          />
        </div>
      )}

      {showIndexName && (
        <div className="flex flex-col gap-1">
          <Label htmlFor="ddl-index-name" className="text-sm font-medium text-foreground">
            {t("schema.ddlIndexName")}
          </Label>
          <Input
            id="ddl-index-name"
            value={indexName}
            onChange={(e) => setIndexName(e.target.value)}
            placeholder="index_name"
          />
        </div>
      )}

      {showIndexConfig && (
        <>
          <div className="flex flex-col gap-1">
            <Label htmlFor="ddl-idx-name" className="text-sm font-medium text-foreground">
              {t("schema.ddlIndexName")}
            </Label>
            <Input
              id="ddl-idx-name"
              value={indexName}
              onChange={(e) => setIndexName(e.target.value)}
              placeholder="idx_name"
            />
          </div>
          <div className="flex flex-col gap-1">
            <Label htmlFor="ddl-idx-columns" className="text-sm font-medium text-foreground">
              {t("schema.ddlIndexColumns")}
            </Label>
            <Input
              id="ddl-idx-columns"
              value={indexColumns}
              onChange={(e) => setIndexColumns(e.target.value)}
              placeholder="col1, col2"
            />
          </div>
          <div className="flex items-center gap-2">
            <Checkbox
              id="ddl-unique"
              checked={unique}
              onCheckedChange={(checked) => setUnique(!!checked)}
            />
            <Label htmlFor="ddl-unique" className="text-sm font-normal">{t("schema.ddlUnique")}</Label>
          </div>
        </>
      )}

      <div className="flex flex-col gap-2">
        <Label className="text-sm font-medium text-foreground">
          {t("schema.ddlPreview")}
        </Label>
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
