import { useState } from "react";

import { useTranslation } from "@/commons/locales/useTranslation";
import { Button } from "@/components/ui/button";

import { FILTER_OPS, VALUELESS_OPS, type ColumnMeta, type FilterOp, type GridFilter, type CellValue } from "../types/data-grid.types";

interface FilterBarProps {
  columns: ColumnMeta[];
  filters: GridFilter[];
  onAddFilter: (filter: GridFilter) => void;
  onRemoveFilter: (index: number) => void;
}

export function FilterBar({ columns, filters, onAddFilter, onRemoveFilter }: FilterBarProps) {
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
    <div className="flex flex-wrap items-center gap-2 border-b border-border bg-card px-3 py-1.5">
      {filters.map((f, i) => (
        <span
          key={i}
          className="inline-flex items-center gap-1 rounded bg-background px-2 py-0.5 text-xs text-foreground"
        >
          {f.column} {opLabel(f.op)} {VALUELESS_OPS.includes(f.op) ? "" : JSON.stringify(f.value)}
          <Button
            type="button"
            variant="ghost"
            size="sm"
            className="ml-0.5 h-auto px-0 py-0 text-xs opacity-60 hover:bg-transparent hover:opacity-100"
            onClick={() => onRemoveFilter(i)}
          >
            &times;
          </Button>
        </span>
      ))}

      <div className="flex items-center gap-1">
        <select
          className="rounded border border-border bg-card px-1.5 py-0.5 text-xs text-foreground"
          value={column}
          onChange={(e) => setColumn(e.target.value)}
        >
          {columns.map((c) => (
            <option key={c.name} value={c.name}>{c.name}</option>
          ))}
        </select>

        <select
          className="rounded border border-border bg-card px-1.5 py-0.5 text-xs text-foreground"
          value={op}
          onChange={(e) => setOp(e.target.value as FilterOp)}
        >
          {FILTER_OPS.map((f) => (
            <option key={f.value} value={f.value}>{f.label}</option>
          ))}
        </select>

        {!VALUELESS_OPS.includes(op) && (
          <input
            className="rounded border border-border bg-card px-1.5 py-0.5 text-xs text-foreground"
            placeholder={t("dataGrid.filter")}
            value={value}
            onChange={(e) => setValue(e.target.value)}
            onKeyDown={(e) => { if (e.key === "Enter") handleAdd(); }}
          />
        )}

        <Button
          type="button"
          variant="ghost"
          size="sm"
          className="h-auto px-2 py-0.5 text-xs text-primary hover:bg-background"
          onClick={handleAdd}
        >
          {t("dataGrid.addFilter")}
        </Button>
      </div>
    </div>
  );
}
