import { useRef, useCallback, useState } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";

import { renderCellValue } from "@/modules/query/types/query.types";
import { Button } from "@/components/ui/button";

import type { CellValue, ColumnMeta, GridSort, Row } from "../types";

/* ------------------------------------------------------------------ */
/*  Props                                                              */
/* ------------------------------------------------------------------ */

export interface UnifiedGridProps {
  /* ---- data ---- */
  columns: ColumnMeta[];
  rows: Row[];

  /* ---- sort ---- */
  sorts: GridSort[];
  onSort: (column: string) => void;

  /* ---- editing (optional) ---- */
  editingCell?: { row: number; col: number } | null;
  onEditCell?: (cell: { row: number; col: number } | null) => void;
  onCellSave?: (rowIdx: number, colIdx: number, value: CellValue) => void;

  /* ---- row actions (optional) ---- */
  onDeleteRow?: (rowIdx: number) => void;
  isDeleting?: boolean;
  canEditRows?: boolean;

  /* ---- column features ---- */
  frozenColumns?: string[];
  hiddenColumns?: string[];
  onToggleFreezeColumn?: (column: string) => void;

  /* ---- loading / empty ---- */
  isLoading?: boolean;
  emptyState?: React.ReactNode;

  /* ---- footer (optional) ---- */
  footer?: React.ReactNode;

  /* ---- extensibility ---- */
  className?: string;
  contentStyle?: React.CSSProperties;
  renderHeaderExtra?: (col: ColumnMeta) => React.ReactNode;
  renderCellEditor?: (cell: CellValue) => React.ReactNode;
  renderJsonCell?: (value: unknown) => React.ReactNode;
}

/* ------------------------------------------------------------------ */
/*  Component                                                          */
/* ------------------------------------------------------------------ */

export function UnifiedGrid({
  columns,
  rows,
  sorts,
  onSort,
  editingCell,
  onEditCell,
  onCellSave,
  onDeleteRow,
  isDeleting,
  canEditRows,
  frozenColumns = [],
  hiddenColumns = [],
  onToggleFreezeColumn,
  isLoading,
  emptyState,
  footer,
  className,
  contentStyle,
  renderHeaderExtra,
  renderCellEditor,
  renderJsonCell,
}: UnifiedGridProps) {
  const parentRef = useRef<HTMLDivElement>(null);
  const [contextMenu, setContextMenu] = useState<{ x: number; y: number; column: string } | null>(null);

  /* ---- virtualization ---- */
  const virtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 32,
    overscan: 10,
  });

  /* ---- column ordering (frozen first, then normal; hidden excluded) ---- */
  const frozenSet = new Set(frozenColumns);
  const hiddenSet = new Set(hiddenColumns);
  const visibleColumns = columns.filter((c) => !hiddenSet.has(c.name));
  const frozenCols = visibleColumns.filter((c) => frozenSet.has(c.name));
  const normalCols = visibleColumns.filter((c) => !frozenSet.has(c.name));
  const orderedColumns = [...frozenCols, ...normalCols];

  const hasRowActions = canEditRows && onDeleteRow != null;
  const extraTrailingCol = hasRowActions ? " 40px" : "";
  const gridStyle: React.CSSProperties = {
    gridTemplateColumns: `40px repeat(${orderedColumns.length}, minmax(120px, 1fr))${extraTrailingCol}`,
  };

  const sortMap = new Map(sorts.map((s) => [s.column, s]));
  const getColumnIndex = (col: ColumnMeta) => columns.findIndex((c) => c.name === col.name);

  /* ---- handlers ---- */
  const handleDoubleClick = useCallback(
    (rowIdx: number, colIdx: number) => {
      if (!canEditRows || !onEditCell) return;
      onEditCell({ row: rowIdx, col: colIdx });
    },
    [canEditRows, onEditCell],
  );

  const handleContextMenu = useCallback(
    (e: React.MouseEvent, columnName: string) => {
      if (!onToggleFreezeColumn) return;
      e.preventDefault();
      setContextMenu({ x: e.clientX, y: e.clientY, column: columnName });
    },
    [onToggleFreezeColumn],
  );

  /* ---- empty state ---- */
  if (!columns.length) {
    return (
      <div className="flex h-full min-h-0 items-center justify-center">
        {emptyState ?? <p className="text-sm text-muted-foreground">No data</p>}
      </div>
    );
  }

  /* ---- render ---- */
  return (
    <div className={`relative flex h-full flex-col${className ? ` ${className}` : ""}`}>
      {/* Loading overlay */}
      {isLoading && (
        <div className="absolute inset-0 z-10 flex items-center justify-center bg-background/60">
          <span className="text-sm text-muted-foreground">Loading…</span>
        </div>
      )}

      {/* Column freeze context menu */}
      {contextMenu && (
        <div
          className="fixed z-[var(--z-floating)] rounded-sm border border-border bg-muted py-1 shadow-lg"
          style={{ left: contextMenu.x, top: contextMenu.y }}
          onMouseLeave={() => setContextMenu(null)}
        >
          <Button
            type="button"
            variant="ghost"
            size="sm"
            className="block h-auto w-full px-3 py-1 text-left text-xs text-foreground hover:bg-background"
            onClick={() => {
              onToggleFreezeColumn?.(contextMenu.column);
              setContextMenu(null);
            }}
          >
            {frozenSet.has(contextMenu.column) ? "Unfreeze" : "Freeze"} "{contextMenu.column}"
          </Button>
        </div>
      )}

      {/* Zoomable content wrapper (header + rows) */}
      <div style={contentStyle} className="flex h-full min-h-0 flex-col">
      {/* Header row */}
      <div
        className="grid shrink-0 border-b border-[var(--app-border)] bg-muted/30 text-[11px] font-medium"
        style={{ ...gridStyle, minHeight: "var(--grid-header-height)" }}
      >
        <div className="flex items-center px-2 py-2 text-[var(--app-text-dim)]">#</div>
        {orderedColumns.map((col) => {
          const sort = sortMap.get(col.name);
          const isFrozen = frozenSet.has(col.name);
          return (
            <div
              key={col.name}
              className={
                "flex flex-col gap-0.5 px-3 py-1.5" +
                (isFrozen ? " bg-muted/30" : "")
              }
              style={isFrozen ? { fontWeight: 600 } : undefined}
              onContextMenu={(e) => handleContextMenu(e, col.name)}
            >
              <span
                className="flex cursor-pointer select-none items-center gap-1 text-[var(--app-text-muted)] transition-colors hover:text-foreground"
                onClick={() => onSort(col.name)}
              >
                <span className="truncate font-medium">{col.name}</span>
                {sort && (
                  <span className="text-[10px]">
                    {sort.direction === "asc" ? "\u25B2" : "\u25BC"}
                  </span>
                )}
              </span>
              <span className="truncate text-[10px] text-[var(--app-text-dim)]">{col.dataType}</span>
              {renderHeaderExtra?.(col)}
            </div>
          );
        })}
        {hasRowActions && <div />}
      </div>

      {/* Virtual rows */}
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
                className="grid absolute w-full border-b border-[var(--app-border-subtle)] text-[12.5px] transition-colors hover:bg-[var(--app-hover)]"
                style={{ ...gridStyle, top: virtualRow.start, height: "var(--grid-row-height)" }}
                data-index={virtualRow.index}
              >
                {/* Row number */}
                <div className="flex items-center px-2 text-[11px] text-[var(--app-text-dim)]">
                  {virtualRow.index + 1}
                </div>

                {/* Cells */}
                {orderedColumns.map((col) => {
                  const colIdx = getColumnIndex(col);
                  const cell = row[colIdx];
                  const display = renderCellValue(cell);
                  const isNull = cell.type === "null";
                  const isJson = cell.type === "json";
                  const isEditing =
                    editingCell?.row === virtualRow.index &&
                    editingCell?.col === colIdx;
                  const isFrozen = frozenSet.has(col.name);

                  return (
                    <div
                      key={col.name}
                      className={
                        "flex items-center overflow-hidden px-3 text-ellipsis whitespace-nowrap" +
                        (isNull ? " text-[var(--app-text-dim)]" : " text-foreground") +
                        (isFrozen ? " bg-muted/20" : "")
                      }
                      style={isNull ? { fontStyle: "italic" } : undefined}
                      title={isJson ? undefined : display}
                      onDoubleClick={() => handleDoubleClick(virtualRow.index, colIdx)}
                    >
                      {isEditing && onCellSave && onEditCell ? (
                        renderCellEditor ? (
                          renderCellEditor(cell)
                        ) : (
                          <InlineCellEditor
                            value={cell}
                            onSave={(newValue) => {
                              onCellSave(virtualRow.index, colIdx, newValue);
                              onEditCell(null);
                            }}
                            onCancel={() => onEditCell(null)}
                          />
                        )
                      ) : isJson && cell.type === "json" ? (
                        renderJsonCell ? (
                          renderJsonCell(cell.value)
                        ) : (
                          <JsonPreview value={cell.value} />
                        )
                      ) : (
                        display
                      )}
                    </div>
                  );
                })}

                {/* Row actions */}
                {hasRowActions && (
                  <div className="flex items-center justify-center px-1">
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      className="h-6 w-6 p-0 text-destructive hover:text-destructive"
                      disabled={isDeleting}
                      onClick={() => onDeleteRow!(virtualRow.index)}
                      title="Delete row"
                    >
                      ×
                    </Button>
                  </div>
                )}
              </div>
            );
          })}
        </div>
      </div>
      </div>

      {/* Footer */}
      {footer && (
        <div className="flex items-center gap-4 border-t border-[var(--app-border-subtle)] bg-muted/20 px-3 py-1.5 text-[11px] text-[var(--app-text-muted)]">
          {footer}
        </div>
      )}
    </div>
  );
}

/* ------------------------------------------------------------------ */
/*  Inline cell editor (minimal)                                       */
/* ------------------------------------------------------------------ */

function InlineCellEditor({
  value,
  onSave,
  onCancel,
}: {
  value: CellValue;
  onSave: (value: CellValue) => void;
  onCancel: () => void;
}) {
  const initialText = value.type === "null" ? "" : String((value as { value: unknown }).value ?? "");
  const [text, setText] = useState(initialText);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      e.preventDefault();
      onSave({ type: "text", value: text });
    } else if (e.key === "Escape") {
      e.preventDefault();
      onCancel();
    }
  };

  return (
    <input
      className="absolute inset-0 z-10 w-full border border-primary bg-background px-2 py-0 text-xs outline-none"
      value={text}
      onChange={(e) => setText(e.target.value)}
      onKeyDown={handleKeyDown}
      onBlur={() => onSave({ type: "text", value: text })}
      autoFocus
    />
  );
}

/* ------------------------------------------------------------------ */
/*  JSON preview                                                       */
/* ------------------------------------------------------------------ */

function JsonPreview({ value }: { value: unknown }) {
  const text = JSON.stringify(value, null, 2);
  const isLong = text.length > 80;
  return (
    <span className="text-muted-foreground" title={text}>
      {isLong ? `${text.slice(0, 80)}…` : text}
    </span>
  );
}
