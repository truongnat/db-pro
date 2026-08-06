import { useState } from "react";

import { useTranslation } from "@/commons/locales/useTranslation";

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
    <div
      className="border-b px-3 py-2"
      style={{ borderColor: "var(--color-border)", backgroundColor: "var(--color-surface)" }}
    >
      {filters.length > 0 && (
        <div className="mb-2 flex flex-wrap gap-1.5">
          {filters.map((f, i) => (
            <span
              key={i}
              className="inline-flex items-center gap-1 rounded-[var(--radius-sm)] px-2 py-1 text-xs"
              style={{
                backgroundColor: "var(--color-primary, #3b82f6)",
                color: "white",
                opacity: 0.85,
              }}
            >
              <span className="font-medium">{f.column}</span>
              <span>{opLabel(f.op)}</span>
              {!VALUELESS_OPS.includes(f.op) && (
                <span>"{VALUELESS_OPS.includes(f.op) ? "" : (f.value as { value?: string }).value ?? ""}"</span>
              )}
              <button
                className="ml-0.5 opacity-70 hover:opacity-100"
                onClick={() => onRemoveFilter(i)}
                type="button"
              >
                ×
              </button>
            </span>
          ))}
        </div>
      )}

      <div className="flex items-center gap-2">
        <select
          className="rounded-[var(--radius-sm)] border px-2 py-1 text-xs"
          style={{
            borderColor: "var(--color-border)",
            backgroundColor: "var(--color-bg)",
            color: "var(--color-text)",
          }}
          value={column}
          onChange={(e) => setColumn(e.target.value)}
        >
          {columns.map((c) => (
            <option key={c.name} value={c.name}>
              {c.name}
            </option>
          ))}
        </select>

        <select
          className="rounded-[var(--radius-sm)] border px-2 py-1 text-xs"
          style={{
            borderColor: "var(--color-border)",
            backgroundColor: "var(--color-bg)",
            color: "var(--color-text)",
          }}
          value={op}
          onChange={(e) => setOp(e.target.value as FilterOp)}
        >
          {FILTER_OPS.map((f) => (
            <option key={f.value} value={f.value}>
              {f.label}
            </option>
          ))}
        </select>

        {!VALUELESS_OPS.includes(op) && (
          <input
            className="rounded-[var(--radius-sm)] border px-2 py-1 text-xs"
            style={{
              borderColor: "var(--color-border)",
              backgroundColor: "var(--color-bg)",
              color: "var(--color-text)",
            }}
            placeholder={t("dataGrid.filter")}
            value={value}
            onChange={(e) => setValue(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") handleAdd();
            }}
          />
        )}

        <button
          className="rounded-[var(--radius-sm)] px-2 py-1 text-xs transition-colors hover:bg-[var(--color-bg)]"
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
