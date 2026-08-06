import type { ChangeEvent } from "react";

import { Button } from "@/components/ui/button";
import { useTranslation } from "@/commons/locales/useTranslation";

import type { ColumnDef } from "../../services/ddl-builder";

interface ColumnDefRowProps {
  column: ColumnDef;
  index: number;
  onChange: (index: number, field: keyof ColumnDef, value: string | boolean) => void;
  onRemove: (index: number) => void;
  canRemove: boolean;
}

export function ColumnDefRow({ column, index, onChange, onRemove, canRemove }: ColumnDefRowProps) {
  const { t } = useTranslation();

  const handleField = (field: keyof ColumnDef) => (e: ChangeEvent<HTMLInputElement>) => {
    onChange(index, field, e.target.value);
  };

  return (
    <div className="grid grid-cols-[1fr_1fr_80px_1fr_40px_32px] items-end gap-2">
      <div className="flex flex-col gap-1">
        {index === 0 && (
          <label className="text-xs text-muted-foreground">
            {t("schema.columnName")}
          </label>
        )}
        <input
          type="text"
          value={column.name}
          onChange={handleField("name")}
          placeholder="column_name"
          className="rounded-sm border border-border bg-background px-2 py-1.5 text-sm text-foreground outline-none focus:border-primary"
        />
      </div>

      <div className="flex flex-col gap-1">
        {index === 0 && (
          <label className="text-xs text-muted-foreground">
            {t("schema.columnDataType")}
          </label>
        )}
        <input
          type="text"
          value={column.dataType}
          onChange={handleField("dataType")}
          placeholder="TEXT"
          className="rounded-sm border border-border bg-background px-2 py-1.5 text-sm text-foreground outline-none focus:border-primary"
        />
      </div>

      <div className="flex flex-col gap-1">
        {index === 0 && (
          <label className="text-xs text-muted-foreground">
            {t("schema.columnNullable")}
          </label>
        )}
        <label className="flex items-center justify-center">
          <input
            type="checkbox"
            checked={column.nullable}
            onChange={(e) => onChange(index, "nullable", e.target.checked)}
            className="h-4 w-4 rounded border border-border accent-primary"
          />
        </label>
      </div>

      <div className="flex flex-col gap-1">
        {index === 0 && (
          <label className="text-xs text-muted-foreground">
            {t("schema.columnDefault")}
          </label>
        )}
        <input
          type="text"
          value={column.defaultValue}
          onChange={handleField("defaultValue")}
          placeholder="DEFAULT"
          className="rounded-sm border border-border bg-background px-2 py-1.5 text-sm text-foreground outline-none focus:border-primary"
        />
      </div>

      <div className="flex flex-col gap-1">
        {index === 0 && (
          <label className="text-xs text-muted-foreground">
            {t("schema.columnPk")}
          </label>
        )}
        <label className="flex items-center justify-center">
          <input
            type="checkbox"
            checked={column.isPk}
            onChange={(e) => onChange(index, "isPk", e.target.checked)}
            className="h-4 w-4 rounded border border-border accent-primary"
          />
        </label>
      </div>

      <Button
        type="button"
        variant="ghost"
        size="icon"
        onClick={() => onRemove(index)}
        disabled={!canRemove}
        aria-label={t("common.actions.delete")}
        title={t("common.actions.delete")}
        className={`h-9 ${canRemove ? "text-destructive" : "text-muted-foreground"}`}
        style={{
          opacity: canRemove ? 1 : 0.3,
        }}
      >
        ×
      </Button>
    </div>
  );
}
