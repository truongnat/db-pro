import { useState } from "react";

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

import {
  FILTER_OPS,
  VALUELESS_OPS,
  type CellValue,
  type ColumnMeta,
  type FilterOp,
  type GridFilter,
} from "../types/data-grid.types";

interface VisualFilterBuilderProps {
  columns: ColumnMeta[];
  filters: GridFilter[];
  onAddFilter: (filter: GridFilter) => void;
  onRemoveFilter: (index: number) => void;
}

export function VisualFilterBuilder({
  columns,
  filters,
  onAddFilter,
  onRemoveFilter,
}: VisualFilterBuilderProps) {
  const { t } = useTranslation();
  const [column, setColumn] = useState(columns[0]?.name ?? "");
  const [op, setOp] = useState<FilterOp>("eq");
  const [value, setValue] = useState("");

  const handleAdd = () => {
    if (!column) return;
    let cellValue: CellValue;
    if (VALUELESS_OPS.includes(op)) {
      cellValue = { type: "null" };
    } else {
      cellValue = { type: "text", value };
    }
    onAddFilter({ column, op, value: cellValue });
    setValue("");
  };

  const opLabel = (o: FilterOp) => FILTER_OPS.find((f) => f.value === o)?.label ?? o;

  return (
    <div className="flex flex-col gap-2.5">
      {filters.length > 0 && (
        <div className="flex flex-wrap gap-1.5">
          {filters.map((f, i) => (
            <span
              key={i}
              className="inline-flex items-center gap-1 rounded-sm bg-primary px-2 py-1 text-xs text-white"
              style={{ opacity: 0.85 }}
            >
              <span className="font-medium">{f.column}</span>
              <span>{opLabel(f.op)}</span>
              {!VALUELESS_OPS.includes(f.op) && (
                <span>
                  "
                  {VALUELESS_OPS.includes(f.op)
                    ? ""
                    : ((f.value as { value?: string }).value ?? "")}
                  "
                </span>
              )}
              <Button
                type="button"
                variant="ghost"
                size="sm"
                className="ml-0.5 h-auto px-0 py-0 text-xs text-white opacity-70 hover:bg-transparent hover:opacity-100"
                onClick={() => onRemoveFilter(i)}
              >
                ×
              </Button>
            </span>
          ))}
        </div>
      )}

      <div className="flex items-center gap-2">
        <Select value={column} onValueChange={setColumn}>
          <SelectTrigger className="w-auto text-xs">
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
          <SelectTrigger className="w-auto text-xs">
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
            className="h-auto px-2 py-1 text-xs"
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
          className="h-auto px-2 py-1 text-xs text-primary hover:bg-background"
          onClick={handleAdd}
        >
          {t("dataGrid.addFilter")}
        </Button>
      </div>
    </div>
  );
}
