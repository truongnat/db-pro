import { useEffect, useState } from "react";
import { Trash2 } from "lucide-react";

import { useTranslation } from "@/commons/locales/useTranslation";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";

import {
  FILTER_OPS,
  VALUELESS_OPS,
  type CellValue,
  type ColumnMeta,
  type FilterOp,
  type GridFilter,
} from "../types/data-grid.types";
import { parseFilterValue } from "../utils/filter-parser";

interface VisualFilterBuilderProps {
  columns: ColumnMeta[];
  draftFilters: GridFilter[];
  onAddDraftFilter: (filter: GridFilter) => void;
  onRemoveDraftFilter: (index: number) => void;
  onApply: () => void;
  onClear: () => void;
  appliedCount: number;
}

export function VisualFilterBuilder({
  columns,
  draftFilters,
  onAddDraftFilter,
  onRemoveDraftFilter,
  onApply,
  onClear,
  appliedCount,
}: VisualFilterBuilderProps) {
  const { t } = useTranslation();
  const [column, setColumn] = useState(columns[0]?.name ?? "");
  const [op, setOp] = useState<FilterOp>("eq");
  const [value, setValue] = useState("");

  useEffect(() => {
    if (columns.length > 0 && !columns.some((c) => c.name === column)) {
      setColumn(columns[0].name);
    }
  }, [columns, column]);

  const selectedColumn = columns.find((c) => c.name === column);

  const handleAdd = () => {
    if (!column || !selectedColumn) return;
    let cellValue: CellValue;
    if (VALUELESS_OPS.includes(op)) {
      cellValue = { type: "null" };
    } else {
      cellValue = parseFilterValue(selectedColumn, value);
    }
    onAddDraftFilter({ column, op, value: cellValue });
    setValue("");
  };

  const hasChanges = draftFilters.length > 0 || appliedCount > 0;

  return (
    <div className="flex flex-col gap-3 p-1">
      {/* Draft filter rows */}
      {draftFilters.length > 0 && (
        <div className="flex flex-col gap-1.5">
          {draftFilters.map((f, i) => (
            <div
              key={i}
              className="flex items-center gap-2 rounded-md bg-muted/50 px-2 py-1.5 text-xs"
            >
              <span className="font-medium text-foreground">{f.column}</span>
              <span className="text-[var(--app-text-muted)]">
                {FILTER_OPS.find((o) => o.value === f.op)?.label ?? f.op}
              </span>
              {!VALUELESS_OPS.includes(f.op) && (
                <span className="text-foreground">
                  "
                  {VALUELESS_OPS.includes(f.op)
                    ? ""
                    : ((f.value as { value?: string }).value ?? "")}
                  "
                </span>
              )}
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    type="button"
                    variant="ghost"
                    size="sm"
                    className="ml-auto h-5 w-5 p-0 text-[var(--app-text-muted)] hover:bg-destructive/10 hover:text-destructive"
                    onClick={() => onRemoveDraftFilter(i)}
                  >
                    <Trash2 className="h-3 w-3" />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>Remove filter</TooltipContent>
              </Tooltip>
            </div>
          ))}
        </div>
      )}

      {/* Add filter row */}
      <div className="flex items-center gap-2">
        <Select value={column} onValueChange={setColumn}>
          <SelectTrigger className="h-7 w-auto text-xs">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {columns.map((c) => (
              <SelectItem key={c.name} value={c.name}>
                {c.name}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>

        <Select value={op} onValueChange={(v) => setOp(v as FilterOp)}>
          <SelectTrigger className="h-7 w-auto text-xs">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {FILTER_OPS.map((f) => (
              <SelectItem key={f.value} value={f.value}>
                {f.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>

        {!VALUELESS_OPS.includes(op) && (
          <Input
            className="h-7 flex-1 px-2 py-1 text-xs"
            placeholder={t("dataGrid.filterValue")}
            value={value}
            onChange={(e) => setValue(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") handleAdd();
            }}
          />
        )}

        <Button
          type="button"
          variant="ghost"
          size="sm"
          className="h-7 shrink-0 px-2 text-xs text-primary hover:bg-background"
          onClick={handleAdd}
        >
          + {t("dataGrid.addFilter")}
        </Button>
      </div>

      {/* Footer: Apply / Clear */}
      <div className="flex items-center justify-between border-t border-[var(--app-border-subtle)] pt-2">
        <Button
          type="button"
          variant="ghost"
          size="sm"
          className="h-7 px-2 text-xs text-[var(--app-text-muted)] hover:bg-background"
          onClick={onClear}
          disabled={!hasChanges}
        >
          {t("common.actions.clear")}
        </Button>
        <Button
          type="button"
          size="sm"
          className="h-7 px-3 text-xs"
          onClick={onApply}
          disabled={draftFilters.length === 0}
        >
          {t("common.actions.apply")}
        </Button>
      </div>
    </div>
  );
}
