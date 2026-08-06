import type { ColumnMeta, GridSort } from "../types/data-grid.types";

interface ColumnHeaderProps {
  column: ColumnMeta;
  sort: GridSort | undefined;
  onSort: (column: string) => void;
}

export function ColumnHeader({ column, sort, onSort }: ColumnHeaderProps) {
  return (
    <div
      className="cursor-pointer select-none px-3 py-2 text-xs font-medium transition-colors hover:bg-[var(--color-bg)]"
      style={{ color: "var(--color-text-secondary)" }}
      onClick={() => onSort(column.name)}
      title={`${column.name} (${column.dataType})`}
    >
      <span>{column.name}</span>
      {sort && (
        <span className="ml-1">
          {sort.direction === "asc" ? "\u25B2" : "\u25BC"}
        </span>
      )}
    </div>
  );
}
