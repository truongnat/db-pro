import { useCallback, useState } from "react";
import { useTranslation } from "@/commons/locales/useTranslation";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

import type { CellValue, ColumnMeta } from "../types/data-grid.types";
import { renderCellValue } from "@/modules/query/types/query.types";
import {
  normalizeColumnType,
  isCellTypeEditable,
  getUnsupportedEditReason,
} from "../utils/column-value-codec";

interface RowEditDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  columns: ColumnMeta[];
  row: CellValue[];
  onSave: (changes: Record<string, CellValue>) => void;
}

interface FieldState {
  value: string;
  isNull: boolean;
  error: string | null;
}

export function RowEditDialog({ open, onOpenChange, columns, row, onSave }: RowEditDialogProps) {
  const { t } = useTranslation();

  const [fields, setFields] = useState<Record<string, FieldState>>(() => {
    const initial: Record<string, FieldState> = {};
    for (let i = 0; i < columns.length; i++) {
      const col = columns[i];
      const cell = row[i];
      initial[col.name] = {
        value: cell?.type === "null" ? "" : renderCellValue(cell ?? { type: "null" }),
        isNull: cell?.type === "null",
        error: null,
      };
    }
    return initial;
  });

  const updateField = useCallback((colName: string, patch: Partial<FieldState>) => {
    setFields((prev) => ({
      ...prev,
      [colName]: { ...prev[colName], ...patch, error: null },
    }));
  }, []);

  const validateAndSave = () => {
    const changes: Record<string, CellValue> = {};
    const nextFields = { ...fields };

    for (const col of columns) {
      const field = fields[col.name];
      const original = row[columns.indexOf(col)];
      const originalText = original?.type === "null" ? "" : renderCellValue(original ?? { type: "null" });

      const cellType = normalizeColumnType(col.dataType);
      if (!isCellTypeEditable(cellType)) continue;

      if (field.isNull) {
        if (original?.type !== "null") {
          changes[col.name] = { type: "null" };
        }
        continue;
      }

      if (field.value === originalText) continue;

      switch (cellType) {
        case "int64": {
          const n = Number(field.value);
          if (!Number.isInteger(n)) {
            nextFields[col.name] = { ...field, error: "Invalid integer" };
            setFields(nextFields);
            return;
          }
          changes[col.name] = { type: "int64", value: n };
          break;
        }
        case "float64": {
          const n = Number(field.value);
          if (Number.isNaN(n)) {
            nextFields[col.name] = { ...field, error: "Invalid number" };
            setFields(nextFields);
            return;
          }
          changes[col.name] = { type: "float64", value: n };
          break;
        }
        case "bool": {
          const lower = field.value.toLowerCase();
          if (lower === "true" || lower === "t" || lower === "1") {
            changes[col.name] = { type: "bool", value: true };
          } else if (lower === "false" || lower === "f" || lower === "0") {
            changes[col.name] = { type: "bool", value: false };
          } else {
            nextFields[col.name] = { ...field, error: "Enter true/false" };
            setFields(nextFields);
            return;
          }
          break;
        }
        case "json": {
          try {
            const parsed = JSON.parse(field.value);
            changes[col.name] = { type: "json", value: parsed };
          } catch {
            nextFields[col.name] = { ...field, error: "Invalid JSON" };
            setFields(nextFields);
            return;
          }
          break;
        }
        case "uuid":
          changes[col.name] = { type: "uuid", value: field.value };
          break;
        case "datetime":
          changes[col.name] = { type: "datetime", value: field.value };
          break;
        default:
          changes[col.name] = { type: "text", value: field.value };
      }
    }

    if (Object.keys(changes).length > 0) {
      onSave(changes);
    }
    onOpenChange(false);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[80vh] max-w-lg overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{t("dataGrid.rowEdit.title")}</DialogTitle>
        </DialogHeader>
        <div className="flex flex-col gap-3 py-2">
          {columns.map((col) => {
            const field = fields[col.name];
            const cellType = normalizeColumnType(col.dataType);
            const editable = isCellTypeEditable(cellType);
            return (
              <div key={col.name} className="flex flex-col gap-1">
                <div className="flex items-center gap-2">
                  <label className="w-32 shrink-0 truncate text-xs font-medium text-foreground" title={col.name}>
                    {col.name}
                  </label>
                  <span className="text-[10px] text-[var(--app-text-dim)]">{col.dataType}</span>
                  <div className="flex-1" />
                  {editable && col.nullable && (
                    <label className="flex items-center gap-1 text-[10px] text-[var(--app-text-muted)]">
                      <input
                        type="checkbox"
                        className="h-3 w-3 accent-primary"
                        checked={field.isNull}
                        onChange={(e) => updateField(col.name, { isNull: e.target.checked })}
                      />
                      NULL
                    </label>
                  )}
                </div>
                {!editable ? (
                  <div className="h-8 rounded bg-muted/30 px-2 py-1 text-xs italic text-[var(--app-text-dim)]">
                    {getUnsupportedEditReason(cellType)}
                  </div>
                ) : field.isNull ? (
                  <div className="h-8 rounded bg-muted/30 px-2 py-1 text-xs italic text-[var(--app-text-dim)]">
                    NULL
                  </div>
                ) : cellType === "bool" ? (
                  <label className="flex items-center gap-2 px-2">
                    <input
                      type="checkbox"
                      className="h-4 w-4 accent-primary"
                      checked={field.value.toLowerCase() === "true" || field.value === "1"}
                      onChange={(e) => updateField(col.name, { value: e.target.checked ? "true" : "false" })}
                    />
                    <span className="text-xs text-foreground">
                      {field.value.toLowerCase() === "true" || field.value === "1" ? "true" : "false"}
                    </span>
                  </label>
                ) : (
                  <Input
                    className={`h-8 text-xs ${field.error ? "border-destructive" : ""}`}
                    value={field.value}
                    onChange={(e) => updateField(col.name, { value: e.target.value })}
                    placeholder={t("dataGrid.rowEdit.enterValue")}
                  />
                )}
                {field.error && (
                  <span className="text-[10px] text-destructive">{field.error}</span>
                )}
              </div>
            );
          })}
        </div>
        <DialogFooter>
          <Button type="button" variant="outline" size="sm" onClick={() => onOpenChange(false)}>
            {t("common.actions.cancel")}
          </Button>
          <Button type="button" size="sm" onClick={validateAndSave}>
            {t("common.actions.save")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
