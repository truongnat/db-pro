import { useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";

import { useTranslation } from "@/commons/locales/useTranslation";

import type { ColumnMeta, Row } from "../types/query.types";
import { renderCellValue } from "../types/query.types";
import type { SortState } from "../state/query.store";

interface ResultGridProps {
  columns: ColumnMeta[];
  rows: Row[];
  sort: SortState;
  onSort: (column: string) => void;
  durationMs: number;
  rowCount: number;
}

export function ResultGrid({
  columns,
  rows,
  sort,
  onSort,
  durationMs,
  rowCount,
}: ResultGridProps) {
  const { t } = useTranslation();
  const parentRef = useRef<HTMLDivElement>(null);

  const virtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 32,
    overscan: 10,
  });

  if (!columns.length) {
    return (
      <div className="flex items-center justify-center py-12">
        <p style={{ color: "var(--color-text-secondary)" }}>
          {t("query.noResults")}
        </p>
      </div>
    );
  }

  const gridStyle: React.CSSProperties = {
    gridTemplateColumns: `40px repeat(${columns.length}, minmax(120px, 1fr))`,
  };

  return (
    <div className="flex h-full flex-col">
      <div
        className="grid border-b text-xs font-medium"
        style={{
          ...gridStyle,
          backgroundColor: "var(--color-surface)",
          borderColor: "var(--color-border)",
        }}
      >
        <div className="px-2 py-2" style={{ color: "var(--color-text-secondary)" }}>
          #
        </div>
        {columns.map((col) => {
          const isSorted = sort.column === col.name;
          return (
            <div
              key={col.name}
              className="cursor-pointer select-none px-3 py-2 transition-colors hover:bg-[var(--color-bg)]"
              style={{ color: "var(--color-text-secondary)" }}
              onClick={() => onSort(col.name)}
              title={`${col.name} (${col.dataType})`}
            >
              <span>{col.name}</span>
              {isSorted && (
                <span className="ml-1">
                  {sort.direction === "asc" ? "\u25B2" : "\u25BC"}
                </span>
              )}
            </div>
          );
        })}
      </div>

      <div ref={parentRef} className="flex-1 overflow-auto">
        <div
          style={{
            height: virtualizer.getTotalSize(),
            width: "100%",
            position: "relative",
          }}
        >
          {virtualizer.getVirtualItems().map((virtualRow) => {
            const row = rows[virtualRow.index];
            return (
              <div
                key={virtualRow.key}
                className="grid absolute w-full border-b text-xs transition-colors hover:bg-[var(--color-surface)]"
                style={{
                  ...gridStyle,
                  top: virtualRow.start,
                  borderColor: "var(--color-border)",
                }}
                data-index={virtualRow.index}
              >
                <div
                  className="px-2 py-1.5"
                  style={{ color: "var(--color-text-secondary)" }}
                >
                  {virtualRow.index + 1}
                </div>
                {row.map((cell, colIdx) => {
                  const display = renderCellValue(cell);
                  const isNull = cell.type === "null";
                  return (
                    <div
                      key={colIdx}
                      className="overflow-hidden px-3 py-1.5 text-ellipsis whitespace-nowrap"
                      style={{
                        color: isNull
                          ? "var(--color-text-secondary)"
                          : "var(--color-text)",
                        fontStyle: isNull ? "italic" : undefined,
                      }}
                      title={display}
                    >
                      {display}
                    </div>
                  );
                })}
              </div>
            );
          })}
        </div>
      </div>

      <div
        className="flex items-center gap-4 border-t px-3 py-1.5 text-xs"
        style={{
          borderColor: "var(--color-border)",
          backgroundColor: "var(--color-surface)",
          color: "var(--color-text-secondary)",
        }}
      >
        <span>{t("query.rowsAffected", { count: rowCount })}</span>
        <span>{t("query.duration", { duration: durationMs })}</span>
      </div>
    </div>
  );
}
