import { useRef, useState } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";

import { Button } from "@/components/ui/button";
import { useTranslation } from "@/commons/locales/useTranslation";

import type { ColumnMeta, Row } from "../types/query.types";
import { renderCellValue } from "../types/query.types";
import type { SortState } from "../state/query.store";
import { ColumnMetadataPopover } from "./column-metadata-popover";
import { ZoomControls } from "./zoom-controls";

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
  const [zoom, setZoom] = useState(100);
  const [metadataColumn, setMetadataColumn] = useState<{ column: ColumnMeta; el: HTMLElement } | null>(null);

  const virtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 32,
    overscan: 10,
  });

  if (!columns.length) {
    return (
      <div className="flex items-center justify-center py-12">
        <p className="text-muted-foreground">
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
      <div style={{ zoom: zoom / 100 }} className="flex h-full flex-col">
      <div
        className="grid border-b border-border bg-card text-xs font-medium"
        style={gridStyle}
      >
        <div className="px-2 py-2 text-muted-foreground">
          #
        </div>
        {columns.map((col) => {
          const isSorted = sort.column === col.name;
          return (
            <div
              key={col.name}
              className="flex items-center gap-1 px-3 py-2 text-muted-foreground"
            >
              <span
                className="cursor-pointer select-none transition-colors hover:bg-background"
                onClick={() => onSort(col.name)}
                title={`${col.name} (${col.dataType})`}
              >
                {col.name}
                {isSorted && (
                  <span className="ml-1">
                    {sort.direction === "asc" ? "\u25B2" : "\u25BC"}
                  </span>
                )}
              </span>
              <Button
                type="button"
                variant="ghost"
                size="sm"
                className="shrink-0 rounded px-1 text-[10px] text-muted-foreground"
                title={t("query.metadata.info")}
                onClick={(e) => {
                  e.stopPropagation();
                  setMetadataColumn({ column: col, el: e.currentTarget });
                }}
              >
                i
              </Button>
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
                className="grid absolute w-full border-b border-border text-xs transition-colors hover:bg-card"
                style={{
                  ...gridStyle,
                  top: virtualRow.start,
                }}
                data-index={virtualRow.index}
              >
                <div
                  className="px-2 py-1.5 text-muted-foreground"
                >
                  {virtualRow.index + 1}
                </div>
                {row.map((cell, colIdx) => {
                  const display = renderCellValue(cell);
                  const isNull = cell.type === "null";
                  return (
                    <div
                      key={colIdx}
                      className={`overflow-hidden px-3 py-1.5 text-ellipsis whitespace-nowrap ${
                        isNull ? "text-muted-foreground" : "text-foreground"
                      }`}
                      style={{ fontStyle: isNull ? "italic" : undefined }}
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
      </div>

      <div className="flex items-center gap-4 border-t border-border bg-card px-3 py-1.5 text-xs text-muted-foreground">
        <span>{t("query.rowsAffected", { count: rowCount })}</span>
        <span>{t("query.duration", { duration: durationMs })}</span>
        <div className="flex-1" />
        <ZoomControls zoom={zoom} onZoomChange={setZoom} />
      </div>

      {metadataColumn && (
        <ColumnMetadataPopover
          column={metadataColumn.column}
          anchorEl={metadataColumn.el}
          onClose={() => setMetadataColumn(null)}
        />
      )}
    </div>
  );
}
