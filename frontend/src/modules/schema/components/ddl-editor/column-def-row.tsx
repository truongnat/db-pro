import type { ChangeEvent } from "react";

import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
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
          <Label className="text-xs text-muted-foreground">
            {t("schema.columnName")}
          </Label>
        )}
        <Input
          value={column.name}
          onChange={handleField("name")}
          placeholder="column_name"
          className="text-sm"
        />
      </div>

      <div className="flex flex-col gap-1">
        {index === 0 && (
          <Label className="text-xs text-muted-foreground">
            {t("schema.columnDataType")}
          </Label>
        )}
        <Input
          value={column.dataType}
          onChange={handleField("dataType")}
          placeholder="TEXT"
          className="text-sm"
        />
      </div>

      <div className="flex flex-col gap-1">
        {index === 0 && (
          <Label className="text-xs text-muted-foreground">
            {t("schema.columnNullable")}
          </Label>
        )}
        <div className="flex items-center justify-center">
          <Checkbox
            checked={column.nullable}
            onCheckedChange={(checked) => onChange(index, "nullable", !!checked)}
          />
        </div>
      </div>

      <div className="flex flex-col gap-1">
        {index === 0 && (
          <Label className="text-xs text-muted-foreground">
            {t("schema.columnDefault")}
          </Label>
        )}
        <Input
          value={column.defaultValue}
          onChange={handleField("defaultValue")}
          placeholder="DEFAULT"
          className="text-sm"
        />
      </div>

      <div className="flex flex-col gap-1">
        {index === 0 && (
          <Label className="text-xs text-muted-foreground">
            {t("schema.columnPk")}
          </Label>
        )}
        <div className="flex items-center justify-center">
          <Checkbox
            checked={column.isPk}
            onCheckedChange={(checked) => onChange(index, "isPk", !!checked)}
          />
        </div>
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
