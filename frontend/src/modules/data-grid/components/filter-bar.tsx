import { useState } from "react";

import { useTranslation } from "@/commons/locales/useTranslation";

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
    <div
      className="flex flex-wrap items-center gap-2 border-b px-3 py-1.5"
      style={{ borderColor: "var(--color-border)", backgroundColor: "var(--color-surface)" }}
    >
      {filters.map((f, i) => (
        <span
          key={i}
          className="inline-flex items-center gap-1 rounded px-2 py-0.5 text-xs"
          style={{ backgroundColor: "var(--color-bg)", color: "var(--color-text)" }}
        >
          {f.column} {opLabel(f.op)} {VALUELESS_OPS.includes(f.op) ? "" : JSON.stringify(f.value)}
          <button
            className="ml-0.5 opacity-60 hover:opacity-100"
            onClick={() => onRemoveFilter(i)}
            type="button"
          >
            &times;
          </button>
        </span>
      ))}

      <div className="flex items-center gap-1">
        <select
          className="rounded border px-1.5 py-0.5 text-xs"
          style={{ borderColor: "var(--color-border)", backgroundColor: "var(--color-surface)", color: "var(--color-text)" }}
          value={column}
          onChange={(e) => setColumn(e.target.value)}
        >
          {columns.map((c) => (
            <option key={c.name} value={c.name}>{c.name}</option>
          ))}
        </select>

        <select
          className="rounded border px-1.5 py-0.5 text-xs"
          style={{ borderColor: "var(--color-border)", backgroundColor: "var(--color-surface)", color: "var(--color-text)" }}
          value={op}
          onChange={(e) => setOp(e.target.value as FilterOp)}
        >
          {FILTER_OPS.map((f) => (
            <option key={f.value} value={f.value}>{f.label}</option>
          ))}
        </select>

        {!VALUELESS_OPS.includes(op) && (
          <input
            className="rounded border px-1.5 py-0.5 text-xs"
            style={{ borderColor: "var(--color-border)", backgroundColor: "var(--color-surface)", color: "var(--color-text)" }}
            placeholder={t("dataGrid.filter")}
            value={value}
            onChange={(e) => setValue(e.target.value)}
            onKeyDown={(e) => { if (e.key === "Enter") handleAdd(); }}
          />
        )}

        <button
          className="rounded-[var(--radius-sm)] px-2 py-0.5 text-xs transition-colors hover:bg-[var(--color-bg)]"
          style={{ color: "var(--color-primary, #3b82f6)" }}
          onClick={handleAdd}
          type="button"
        >
          {t("dataGrid.addFilter")}
        </button>
      </div>
    </div>
  );
}
