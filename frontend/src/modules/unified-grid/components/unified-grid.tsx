import { useRef, useCallback, useState, useEffect } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";

import { renderCellValue } from "@/modules/query/types/query.types";
import { Button } from "@/components/ui/button";

import type { CellValue, ColumnMeta, GridSort, Row } from "../types";

/* ------------------------------------------------------------------ */
/*  Constants                                                          */
/* ------------------------------------------------------------------ */

const DEFAULT_COL_WIDTH = 150;
const MIN_COL_WIDTH = 60;
const ROW_NUMBER_WIDTH = 40;
const ROW_ACTION_WIDTH = 40;

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

  /* ---- column widths (B1.1) ---- */
  columnWidths?: Record<string, number>;
  onColumnWidthsChange?: (widths: Record<string, number>) => void;

  /* ---- row selection (B1.3) ---- */
  selectedRows?: Set<number>;
  onSelectionChange?: (selected: Set<number>) => void;

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
  columnWidths: controlledWidths,
  onColumnWidthsChange,
  selectedRows: controlledSelection,
  onSelectionChange,
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
  const gridContainerRef = useRef<HTMLDivElement>(null);

  /* ---- context menu (column + cell) ---- */
  const [contextMenu, setContextMenu] = useState<{
    x: number; y: number; column: string;
    cellRow?: number; cellCol?: number;
  } | null>(null);

  /* ---- column widths (B1.1) ---- */
  const [internalWidths, setInternalWidths] = useState<Record<string, number>>({});
  const widths = controlledWidths ?? internalWidths;
  const setWidths = useCallback(
    (next: Record<string, number>) => {
      if (onColumnWidthsChange) onColumnWidthsChange(next);
      else setInternalWidths(next);
    },
    [onColumnWidthsChange],
  );

  /* ---- resize state (B1.1) ---- */
  const resizing = useRef<{ col: string; startX: number; startW: number } | null>(null);

  const handleResizeStart = useCallback(
    (e: React.MouseEvent, colName: string) => {
      e.preventDefault();
      e.stopPropagation();
      const currentW = widths[colName] ?? DEFAULT_COL_WIDTH;
      resizing.current = { col: colName, startX: e.clientX, startW: currentW };

      const onMove = (ev: MouseEvent) => {
        if (!resizing.current) return;
        const delta = ev.clientX - resizing.current.startX;
        const newW = Math.max(MIN_COL_WIDTH, resizing.current.startW + delta);
        setWidths({ ...widths, [resizing.current.col]: newW });
      };
      const onUp = () => {
        resizing.current = null;
        document.removeEventListener("mousemove", onMove);
        document.removeEventListener("mouseup", onUp);
        document.body.style.cursor = "";
        document.body.style.userSelect = "";
      };
      document.body.style.cursor = "col-resize";
      document.body.style.userSelect = "none";
      document.addEventListener("mousemove", onMove);
      document.addEventListener("mouseup", onUp);
    },
    [widths, setWidths],
  );

  /* ---- row selection (B1.3) ---- */
  const [internalSelection, setInternalSelection] = useState<Set<number>>(new Set());
  const selection = controlledSelection ?? internalSelection;
  const setSelection = useCallback(
    (next: Set<number>) => {
      if (onSelectionChange) onSelectionChange(next);
      else setInternalSelection(next);
    },
    [onSelectionChange],
  );
  const lastSelectedRow = useRef<number | null>(null);

  /* ---- clear selection when dataset changes (pagination/sort/filter/refresh) ---- */
  const prevRowsRef = useRef(rows);
  useEffect(() => {
    if (prevRowsRef.current !== rows) {
      setSelection(new Set());
      lastSelectedRow.current = null;
      prevRowsRef.current = rows;
    }
  }, [rows, setSelection]);

  const handleRowNumberClick = useCallback(
    (e: React.MouseEvent, rowIdx: number) => {
      e.stopPropagation();
      if (e.shiftKey && lastSelectedRow.current != null) {
        const from = Math.min(lastSelectedRow.current, rowIdx);
        const to = Math.max(lastSelectedRow.current, rowIdx);
        const next = new Set(selection);
        for (let i = from; i <= to; i++) next.add(i);
        setSelection(next);
      } else if (e.ctrlKey || e.metaKey) {
        const next = new Set(selection);
        if (next.has(rowIdx)) next.delete(rowIdx);
        else next.add(rowIdx);
        setSelection(next);
      } else {
        setSelection(new Set([rowIdx]));
      }
      lastSelectedRow.current = rowIdx;
    },
    [selection, setSelection],
  );

  /* ---- virtualization ---- */
  const virtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 30,
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
  const extraTrailingCol = hasRowActions ? ` ${ROW_ACTION_WIDTH}px` : "";

  /* ---- grid template from widths (B1.1) ---- */
  const colWidthPx = orderedColumns.map((c) => `${widths[c.name] ?? DEFAULT_COL_WIDTH}px`).join(" ");
  const gridStyle: React.CSSProperties = {
    gridTemplateColumns: `${ROW_NUMBER_WIDTH}px ${colWidthPx}${extraTrailingCol}`,
  };

  const sortMap = new Map(sorts.map((s) => [s.column, s]));
  const getColumnIndex = (col: ColumnMeta) => columns.findIndex((c) => c.name === col.name);

  /* ---- keyboard copy (B1.2 / B1.3) ---- */
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "c" && selection.size > 0) {
        const selectedIndices = Array.from(selection).sort((a, b) => a - b);
        const lines = selectedIndices.map((rowIdx) => {
          const row = rows[rowIdx];
          if (!row) return "";
          return orderedColumns.map((col) => {
            const colIdx = getColumnIndex(col);
            return renderCellValue(row[colIdx]);
          }).join("\t");
        });
        navigator.clipboard.writeText(lines.join("\n")).catch(() => {});
      }
    };
    document.addEventListener("keydown", handler);
    return () => document.removeEventListener("keydown", handler);
  }, [selection, rows, orderedColumns, columns]);

  /* ---- handlers ---- */
  const handleDoubleClick = useCallback(
    (rowIdx: number, colIdx: number) => {
      if (!canEditRows || !onEditCell) return;
      onEditCell({ row: rowIdx, col: colIdx });
    },
    [canEditRows, onEditCell],
  );

  const handleColumnContextMenu = useCallback(
    (e: React.MouseEvent, columnName: string) => {
      e.preventDefault();
      e.stopPropagation();
      setContextMenu({ x: e.clientX, y: e.clientY, column: columnName });
    },
    [],
  );

  const handleCellContextMenu = useCallback(
    (e: React.MouseEvent, columnName: string, rowIdx: number, colIdx: number) => {
      e.preventDefault();
      e.stopPropagation();
      setContextMenu({ x: e.clientX, y: e.clientY, column: columnName, cellRow: rowIdx, cellCol: colIdx });
    },
    [],
  );

  /* ---- copy helpers (B1.2) ---- */
  const copyCellValue = (rowIdx: number, colName: string) => {
    const row = rows[rowIdx];
    if (!row) return;
    const colIdx = getColumnIndex(orderedColumns.find((c) => c.name === colName)!);
    const cell = row[colIdx];
    navigator.clipboard.writeText(renderCellValue(cell)).catch(() => {});
  };

  const copyRowValues = (rowIdx: number) => {
    const row = rows[rowIdx];
    if (!row) return;
    const text = orderedColumns.map((col) => {
      const colIdx = getColumnIndex(col);
      return renderCellValue(row[colIdx]);
    }).join("\t");
    navigator.clipboard.writeText(text).catch(() => {});
  };

  const copyColumnName = (colName: string) => {
    navigator.clipboard.writeText(colName).catch(() => {});
  };

  /* ---- empty state ---- */
  if (!columns.length) {
    return (
      <div className="flex h-full min-h-0 items-center justify-center">
        {emptyState ?? <p className="text-sm text-[var(--app-text-muted)]">No data</p>}
      </div>
    );
  }

  /* ---- render ---- */
  return (
    <div ref={gridContainerRef} className={`relative flex h-full flex-col${className ? ` ${className}` : ""}`}>
      {/* Loading overlay */}
      {isLoading && (
        <div className="absolute inset-0 z-10 flex items-center justify-center bg-background/60">
          <span className="text-sm text-[var(--app-text-muted)]">Loading…</span>
        </div>
      )}

      {/* Context menu (B1.2 — extended with copy items) */}
      {contextMenu && (
        <div
          className="fixed z-[var(--z-floating)] rounded-md border border-[var(--app-border-strong)] bg-popover py-1 shadow-lg"
          style={{ left: contextMenu.x, top: contextMenu.y }}
          onMouseLeave={() => setContextMenu(null)}
        >
          {/* Column actions */}
          {onToggleFreezeColumn && (
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
              {frozenSet.has(contextMenu.column) ? "Restore order" : "Move to front"} "{contextMenu.column}"
            </Button>
          )}
          <Button
            type="button"
            variant="ghost"
            size="sm"
            className="block h-auto w-full px-3 py-1 text-left text-xs text-foreground hover:bg-background"
            onClick={() => { copyColumnName(contextMenu.column); setContextMenu(null); }}
          >
            Copy column name
          </Button>
          {/* Cell / row copy (only when right-clicked on a cell) */}
          {contextMenu.cellRow != null && (
            <>
              <div className="my-1 border-t border-[var(--app-border-subtle)]" />
              <Button
                type="button"
                variant="ghost"
                size="sm"
                className="block h-auto w-full px-3 py-1 text-left text-xs text-foreground hover:bg-background"
                onClick={() => { copyCellValue(contextMenu.cellRow!, contextMenu.column); setContextMenu(null); }}
              >
                Copy cell value
              </Button>
              <Button
                type="button"
                variant="ghost"
                size="sm"
                className="block h-auto w-full px-3 py-1 text-left text-xs text-foreground hover:bg-background"
                onClick={() => { copyRowValues(contextMenu.cellRow!); setContextMenu(null); }}
              >
                Copy row
              </Button>
            </>
          )}
        </div>
      )}

      {/* Zoomable content wrapper (header + rows) */}
      <div style={contentStyle} className="flex h-full min-h-0 flex-col">
      {/* Header row */}
      <div
        className="grid shrink-0 border-b border-[var(--app-border)] bg-muted/30 text-[11px]"
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
                "relative flex flex-col gap-0.5 px-3 py-1.5" +
                (isFrozen ? " bg-muted/30" : "")
              }
              style={isFrozen ? { fontWeight: 600 } : undefined}
              onContextMenu={(e) => handleColumnContextMenu(e, col.name)}
            >
              <span
                className="flex cursor-pointer select-none items-center gap-1 text-[12.5px] font-medium text-[var(--app-text-muted)] transition-colors hover:text-foreground"
                onClick={() => onSort(col.name)}
              >
                <span className="truncate">{col.name}</span>
                {sort && (
                  <span className="text-[11px] text-[var(--app-text-dim)]">
                    {sort.direction === "asc" ? "\u25B2" : "\u25BC"}
                  </span>
                )}
              </span>
              <span className="truncate text-[11px] text-[var(--app-text-dim)]">{col.dataType}</span>
              {renderHeaderExtra?.(col)}
              {/* Resize handle (B1.1) */}
              <div
                className="absolute right-0 top-0 h-full w-[3px] cursor-col-resize hover:bg-primary/40 active:bg-primary"
                onMouseDown={(e) => handleResizeStart(e, col.name)}
              />
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
            const isSelected = selection.has(virtualRow.index);
            return (
              <div
                key={virtualRow.key}
                className={
                  "grid absolute w-full border-b border-[var(--app-border-subtle)]/60 text-[12.5px] transition-colors" +
                  (isSelected ? " bg-primary/10" : " hover:bg-[var(--app-hover)]")
                }
                style={{ ...gridStyle, top: virtualRow.start, height: "var(--grid-row-height)" }}
                data-index={virtualRow.index}
              >
                {/* Row number — clickable for selection (B1.3) */}
                <div
                  className={
                    "flex cursor-pointer select-none items-center px-2 text-[11px]" +
                    (isSelected ? " bg-primary/20 font-medium text-primary" : " text-[var(--app-text-dim)]")
                  }
                  onClick={(e) => handleRowNumberClick(e, virtualRow.index)}
                >
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
                      onContextMenu={(e) => handleCellContextMenu(e, col.name, virtualRow.index, colIdx)}
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
  const [expanded, setExpanded] = useState(false);
  const text = JSON.stringify(value, null, 2);
  const isLong = text.length > 80;

  if (isLong && !expanded) {
    return (
      <button
        type="button"
        className="cursor-text text-left text-[var(--app-text-muted)] hover:text-foreground"
        onClick={(e) => {
          e.stopPropagation();
          setExpanded(true);
        }}
        title={text}
      >
        {"{\u2026}"}
      </button>
    );
  }

  if (isLong && expanded) {
    return (
      <button
        type="button"
        className="cursor-text whitespace-pre-wrap text-left text-[var(--app-text-muted)] hover:text-foreground"
        onClick={(e) => {
          e.stopPropagation();
          setExpanded(false);
        }}
      >
        {text}
      </button>
    );
  }

  return (
    <span className="text-[var(--app-text-muted)]" title={text}>
      {text}
    </span>
  );
}
