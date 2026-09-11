import { useRef, useCallback, useState, useEffect } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import {
  ArrowDown,
  ArrowUp,
  ArrowUpDown,
  Check,
  Loader2,
  Minus,
  Pencil,
  Trash2,
} from "lucide-react";

import { renderCellValue } from "@/modules/query/types/query.types";
import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { useTranslation } from "@/commons/locales/useTranslation";
import {
  normalizeColumnType,
  isCellTypeEditable,
} from "@/modules/data-grid/utils/column-value-codec";

import type { CellValue, ColumnMeta, GridSort, Row } from "../types";

/* ------------------------------------------------------------------ */
/*  Constants                                                          */
/* ------------------------------------------------------------------ */

const DEFAULT_COL_WIDTH = 150;
const MIN_COL_WIDTH = 60;
const ROW_NUMBER_WIDTH = 40;
const ROW_ACTION_WIDTH = 40;
const CONTEXT_MENU_WIDTH = 220;
const CONTEXT_MENU_HEIGHT = 180;
const SKELETON_ROW_COUNT = 8;

/* ------------------------------------------------------------------ */
/*  Cell presentation                                                  */
/* ------------------------------------------------------------------ */

/**
 * Numeric cells are right-aligned with tabular figures so columns of values
 * can be scanned vertically — the single biggest readability win in a data
 * grid, and the behaviour every desktop DB client (DBeaver, DataGrip, pgAdmin)
 * ships by default.
 */
function isNumericCell(cell: CellValue | undefined): boolean {
  return cell?.type === "int64" || cell?.type === "float64";
}

function cellAlignmentClass(cell: CellValue | undefined): string {
  return isNumericCell(cell) ? "justify-end tabular-nums" : "";
}

/**
 * NULL / boolean sentinels are rendered as quiet, unmistakable markers rather
 * than plain text, so an empty string can never be mistaken for SQL NULL.
 */
function CellValueView({ cell, display }: { cell: CellValue; display: string }) {
  if (cell.type === "null") {
    return (
      <span
        className="rounded-sm bg-[var(--surface-hover)] px-1 py-px text-[11px] italic text-[var(--text-tertiary)]"
        data-cell-kind="null"
      >
        NULL
      </span>
    );
  }

  if (cell.type === "bool") {
    return (
      <span
        className="inline-flex items-center gap-1"
        data-cell-kind={cell.value ? "true" : "false"}
      >
        {cell.value ? (
          <Check className="h-3 w-3 shrink-0 text-[var(--state-success)]" aria-hidden="true" />
        ) : (
          <Minus className="h-3 w-3 shrink-0 text-[var(--text-tertiary)]" aria-hidden="true" />
        )}
        <span className={cell.value ? "text-foreground" : "text-[var(--text-tertiary)]"}>
          {display}
        </span>
      </span>
    );
  }

  return <span className="truncate">{display}</span>;
}

/** Placeholder rows shown while a page/result set is still in flight. */
function GridSkeleton({ columns }: { columns: number }) {
  return (
    <div className="flex flex-col" aria-hidden="true">
      {Array.from({ length: SKELETON_ROW_COUNT }, (_, rowIndex) => (
        <div
          key={rowIndex}
          className="flex items-center gap-3 border-b border-[var(--border-subtle)]/60 px-3"
          style={{ height: "var(--grid-row-height)" }}
        >
          {Array.from({ length: Math.min(columns, 8) }, (_, colIndex) => (
            <span
              key={colIndex}
              className="h-2.5 animate-pulse rounded-sm bg-[var(--surface-hover)]"
              style={{ width: `${38 + ((rowIndex * 7 + colIndex * 13) % 45)}%` }}
            />
          ))}
        </div>
      ))}
    </div>
  );
}

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
  onEditRow?: (rowIdx: number) => void;
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
  renderCellEditor?: (cell: CellValue, columnName: string) => React.ReactNode;
  renderJsonCell?: (value: unknown) => React.ReactNode;
  onKeyDown?: (e: React.KeyboardEvent) => void;
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
  onEditRow,
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
  onKeyDown: externalKeyDown,
}: UnifiedGridProps) {
  const { t } = useTranslation();
  const parentRef = useRef<HTMLDivElement>(null);
  const gridContainerRef = useRef<HTMLDivElement>(null);

  /* ---- context menu (column + cell) ---- */
  const [contextMenu, setContextMenu] = useState<{
    x: number;
    y: number;
    column: string;
    cellRow?: number;
    cellCol?: number;
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
  const resizeCleanup = useRef<(() => void) | null>(null);

  useEffect(() => {
    return () => resizeCleanup.current?.();
  }, []);

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
        resizeCleanup.current = null;
        document.removeEventListener("mousemove", onMove);
        document.removeEventListener("mouseup", onUp);
        document.body.style.cursor = "";
        document.body.style.userSelect = "";
      };
      resizeCleanup.current = onUp;
      document.body.style.cursor = "col-resize";
      document.body.style.userSelect = "none";
      document.addEventListener("mousemove", onMove);
      document.addEventListener("mouseup", onUp);
    },
    [widths, setWidths],
  );

  const handleResizeKeyDown = useCallback(
    (e: React.KeyboardEvent, colName: string) => {
      if (e.key !== "ArrowLeft" && e.key !== "ArrowRight") return;
      e.preventDefault();
      const currentW = widths[colName] ?? DEFAULT_COL_WIDTH;
      const delta = e.key === "ArrowRight" ? 10 : -10;
      setWidths({ ...widths, [colName]: Math.max(MIN_COL_WIDTH, currentW + delta) });
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
  const contextMenuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (contextMenu) contextMenuRef.current?.focus();
  }, [contextMenu]);

  useEffect(() => {
    if (!contextMenu) return;
    const handleOutsideClick = (e: MouseEvent) => {
      if (contextMenuRef.current && !contextMenuRef.current.contains(e.target as Node)) {
        setContextMenu(null);
      }
    };
    document.addEventListener("mousedown", handleOutsideClick);
    return () => document.removeEventListener("mousedown", handleOutsideClick);
  }, [contextMenu]);

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
    (e: React.MouseEvent | React.KeyboardEvent, rowIdx: number) => {
      e.stopPropagation();
      // Establish grid keyboard ownership so Cmd/Ctrl+C works after selection.
      // Skip if user is actively editing a cell (input inside grid container).
      const active = document.activeElement;
      if (!(active instanceof HTMLInputElement || active instanceof HTMLTextAreaElement)) {
        gridContainerRef.current?.focus({ preventScroll: true });
      }
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

  /* ---- sticky left offsets for pinned columns (gutter first) ---- */
  const stickyOffsets = new Map<string, number>();
  {
    let runningLeft = ROW_NUMBER_WIDTH;
    for (const col of frozenCols) {
      stickyOffsets.set(col.name, runningLeft);
      runningLeft += widths[col.name] ?? DEFAULT_COL_WIDTH;
    }
  }

  const hasRowActions = canEditRows && (onDeleteRow != null || onEditRow != null);
  const rowActionCount = (onDeleteRow ? 1 : 0) + (onEditRow ? 1 : 0);
  const rowActionsWidth = rowActionCount * ROW_ACTION_WIDTH;
  const extraTrailingCol = hasRowActions ? ` ${rowActionsWidth}px` : "";

  /* ---- grid template from widths (B1.1) ---- */
  const colWidthPx = orderedColumns
    .map((c) => `${widths[c.name] ?? DEFAULT_COL_WIDTH}px`)
    .join(" ");
  const gridStyle: React.CSSProperties = {
    gridTemplateColumns: `${ROW_NUMBER_WIDTH}px ${colWidthPx}${extraTrailingCol}`,
  };

  const sortMap = new Map(sorts.map((s) => [s.column, s]));
  const getColumnIndex = (col: ColumnMeta) => columns.findIndex((c) => c.name === col.name);

  /* ---- keyboard copy (B1.2 / B1.3) — scoped to grid focus ---- */
  const handleGridKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      // Let external handler run first (e.g. Cmd+Enter to apply)
      externalKeyDown?.(e);
      if (e.defaultPrevented) return;

      // Do not intercept copy when focus is inside an editable element
      const target = e.target as HTMLElement;
      if (
        target instanceof HTMLInputElement ||
        target instanceof HTMLTextAreaElement ||
        target instanceof HTMLSelectElement ||
        target.isContentEditable
      ) {
        return;
      }
      if ((e.ctrlKey || e.metaKey) && e.key === "c" && selection.size > 0) {
        e.preventDefault();
        const selectedIndices = Array.from(selection).sort((a, b) => a - b);
        const lines = selectedIndices.map((rowIdx) => {
          const row = rows[rowIdx];
          if (!row) return "";
          return orderedColumns
            .map((col) => {
              const colIdx = getColumnIndex(col);
              return renderCellValue(row[colIdx]);
            })
            .join("\t");
        });
        navigator.clipboard.writeText(lines.join("\n")).catch(() => {});
      }
    },
    [selection, rows, orderedColumns, columns, externalKeyDown],
  );

  /* ---- handlers ---- */
  const handleDoubleClick = useCallback(
    (rowIdx: number, colIdx: number) => {
      if (!canEditRows || !onEditCell) return;
      const col = columns[colIdx];
      if (col && !isCellTypeEditable(normalizeColumnType(col.dataType))) return;
      onEditCell({ row: rowIdx, col: colIdx });
    },
    [canEditRows, onEditCell, columns],
  );

  const handleColumnContextMenu = useCallback((e: React.MouseEvent, columnName: string) => {
    e.preventDefault();
    e.stopPropagation();
    setContextMenu({
      x: Math.min(Math.max(4, e.clientX), window.innerWidth - CONTEXT_MENU_WIDTH - 4),
      y: Math.min(Math.max(4, e.clientY), window.innerHeight - CONTEXT_MENU_HEIGHT - 4),
      column: columnName,
    });
  }, []);

  const handleCellContextMenu = useCallback(
    (e: React.MouseEvent, columnName: string, rowIdx: number, colIdx: number) => {
      e.preventDefault();
      e.stopPropagation();
      setContextMenu({
        x: Math.min(Math.max(4, e.clientX), window.innerWidth - CONTEXT_MENU_WIDTH - 4),
        y: Math.min(Math.max(4, e.clientY), window.innerHeight - CONTEXT_MENU_HEIGHT - 4),
        column: columnName,
        cellRow: rowIdx,
        cellCol: colIdx,
      });
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
    const text = orderedColumns
      .map((col) => {
        const colIdx = getColumnIndex(col);
        return renderCellValue(row[colIdx]);
      })
      .join("\t");
    navigator.clipboard.writeText(text).catch(() => {});
  };

  const copyColumnName = (colName: string) => {
    navigator.clipboard.writeText(colName).catch(() => {});
  };

  /* ---- empty state ---- */
  if (!columns.length) {
    return (
      <div className="flex h-full min-h-0 items-center justify-center">
        {emptyState ?? (
          <p className="text-sm text-[var(--text-secondary)]">{t("dataGrid.noData")}</p>
        )}
      </div>
    );
  }

  /* ---- render ---- */
  return (
    <div
      ref={gridContainerRef}
      tabIndex={0}
      onKeyDown={handleGridKeyDown}
      className={`relative flex h-full flex-col${className ? ` ${className}` : ""}`}
      role="grid"
      aria-rowcount={rows.length}
      aria-colcount={orderedColumns.length}
      aria-busy={isLoading || undefined}
      aria-label={t("dataGrid.gridAriaLabel")}
    >
      {/* Loading overlay — skeleton rows plus a live-region label */}
      {isLoading && (
        <div className="absolute inset-0 z-10 bg-background/70">
          <GridSkeleton columns={orderedColumns.length} />
          <div
            role="status"
            className="absolute inset-x-0 top-1/2 flex -translate-y-1/2 items-center justify-center gap-2 text-[12px] text-[var(--text-secondary)]"
          >
            <Loader2 className="h-3.5 w-3.5 animate-spin text-primary" aria-hidden="true" />
            {t("common.states.loading")}
          </div>
        </div>
      )}

      {/* Context menu (B1.2 — extended with copy items) */}
      {contextMenu && (
        <div
          ref={contextMenuRef}
          role="menu"
          tabIndex={-1}
          aria-label={t("dataGrid.contextMenuLabel")}
          className="fixed z-[var(--z-floating)] rounded-md border border-[var(--border-strong)] bg-popover py-1 shadow-lg"
          style={{ left: contextMenu.x, top: contextMenu.y }}
          onKeyDown={(e) => {
            if (e.key === "Escape") setContextMenu(null);
          }}
        >
          {/* Column actions */}
          {onToggleFreezeColumn && (
            <Button
              type="button"
              variant="ghost"
              size="sm"
              className="block h-auto w-full px-3 py-1 text-left text-xs text-foreground hover:bg-[var(--surface-hover)] focus-visible:bg-[var(--surface-hover)]"
              role="menuitem"
              onClick={() => {
                onToggleFreezeColumn?.(contextMenu.column);
                setContextMenu(null);
              }}
            >
              {frozenSet.has(contextMenu.column)
                ? t("dataGrid.restoreOrder")
                : t("dataGrid.moveToFront")}{" "}
              "{contextMenu.column}"
            </Button>
          )}
          <Button
            type="button"
            variant="ghost"
            size="sm"
            className="block h-auto w-full px-3 py-1 text-left text-xs text-foreground hover:bg-[var(--surface-hover)] focus-visible:bg-[var(--surface-hover)]"
            role="menuitem"
            onClick={() => {
              copyColumnName(contextMenu.column);
              setContextMenu(null);
            }}
          >
            {t("dataGrid.copyColumnName")}
          </Button>
          {/* Cell / row copy (only when right-clicked on a cell) */}
          {contextMenu.cellRow != null && (
            <>
              <div className="my-1 border-t border-[var(--border-subtle)]" />
              <Button
                type="button"
                variant="ghost"
                size="sm"
                className="block h-auto w-full px-3 py-1 text-left text-xs text-foreground hover:bg-[var(--surface-hover)] focus-visible:bg-[var(--surface-hover)]"
                role="menuitem"
                onClick={() => {
                  copyCellValue(contextMenu.cellRow!, contextMenu.column);
                  setContextMenu(null);
                }}
              >
                {t("dataGrid.copyCellValue")}
              </Button>
              <Button
                type="button"
                variant="ghost"
                size="sm"
                className="block h-auto w-full px-3 py-1 text-left text-xs text-foreground hover:bg-[var(--surface-hover)] focus-visible:bg-[var(--surface-hover)]"
                role="menuitem"
                onClick={() => {
                  copyRowValues(contextMenu.cellRow!);
                  setContextMenu(null);
                }}
              >
                {t("dataGrid.copyRow")}
              </Button>
            </>
          )}
        </div>
      )}

      {/* Grid viewport — single scroll owner for both axes */}
      <div
        ref={parentRef}
        style={contentStyle}
        className="flex h-full min-h-0 flex-col overflow-auto"
      >
        {/* Sticky header — scrolls horizontally with body, stays on top vertically */}
        <div
          className="grid sticky top-0 z-[2] shrink-0 border-b border-[var(--border-default)] bg-muted text-[11px]"
          style={{ ...gridStyle, minHeight: "var(--grid-header-height)" }}
          role="row"
        >
          <div
            className="flex items-center border-r border-[var(--border-subtle)] bg-muted px-2 py-2 text-[var(--text-tertiary)]"
            style={{ position: "sticky", left: 0, zIndex: 3 }}
            role="columnheader"
            aria-label={t("dataGrid.rowNumberColumn")}
          >
            {onSelectionChange && (
              <input
                type="checkbox"
                className="h-3 w-3 cursor-pointer accent-primary"
                aria-label={t("dataGrid.selectAllRows")}
                checked={selection.size === rows.length && rows.length > 0}
                ref={(el) => {
                  if (el) el.indeterminate = selection.size > 0 && selection.size < rows.length;
                }}
                onChange={() => {
                  if (selection.size === rows.length) setSelection(new Set());
                  else setSelection(new Set(rows.map((_, i) => i)));
                }}
              />
            )}
            {!onSelectionChange && "#"}
          </div>
          {orderedColumns.map((col, colIndex) => {
            const sort = sortMap.get(col.name);
            const isFrozen = frozenSet.has(col.name);
            const frozenOffset = isFrozen ? stickyOffsets.get(col.name) : undefined;
            const isLastFrozen = isFrozen && colIndex === frozenCols.length - 1;
            return (
              <div
                key={col.name}
                className={
                  "group/header relative flex flex-col gap-0.5 bg-muted px-3 py-1.5" +
                  (isFrozen ? " font-semibold" : "") +
                  (isLastFrozen ? " border-r border-[var(--border-default)]" : "")
                }
                style={
                  frozenOffset === undefined
                    ? undefined
                    : { position: "sticky", left: frozenOffset, zIndex: 3 }
                }
                onContextMenu={(e) => handleColumnContextMenu(e, col.name)}
                role="columnheader"
                aria-sort={sort ? (sort.direction === "asc" ? "ascending" : "descending") : "none"}
              >
                <span
                  className="flex cursor-pointer select-none items-center gap-1 text-[12.5px] font-medium text-[var(--text-secondary)] transition-colors hover:text-foreground"
                  onClick={() => onSort(col.name)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter" || e.key === " ") {
                      e.preventDefault();
                      onSort(col.name);
                    }
                  }}
                  role="button"
                  tabIndex={0}
                  aria-label={t("dataGrid.sortBy", { column: col.name })}
                >
                  <span className="truncate">{col.name}</span>
                  {sort ? (
                    sort.direction === "asc" ? (
                      <ArrowUp className="h-3 w-3 shrink-0 text-primary" aria-hidden="true" />
                    ) : (
                      <ArrowDown className="h-3 w-3 shrink-0 text-primary" aria-hidden="true" />
                    )
                  ) : (
                    <ArrowUpDown
                      className="h-3 w-3 shrink-0 text-[var(--text-tertiary)] opacity-0 transition-opacity group-hover/header:opacity-60"
                      aria-hidden="true"
                    />
                  )}
                </span>
                <span className="truncate font-mono text-[11px] text-[var(--text-tertiary)]">
                  {col.dataType}
                </span>
                {renderHeaderExtra?.(col)}
                {/* Resize handle (B1.1) */}
                <div
                  className="absolute right-0 top-0 h-full w-[3px] cursor-col-resize hover:bg-primary/40 focus-visible:bg-primary active:bg-primary"
                  onMouseDown={(e) => handleResizeStart(e, col.name)}
                  onKeyDown={(e) => handleResizeKeyDown(e, col.name)}
                  role="separator"
                  tabIndex={0}
                  aria-label={t("dataGrid.resizeColumn", { column: col.name })}
                  aria-valuemin={MIN_COL_WIDTH}
                  aria-valuenow={widths[col.name] ?? DEFAULT_COL_WIDTH}
                  aria-orientation="vertical"
                />
              </div>
            );
          })}
          {hasRowActions && <div role="columnheader" aria-label={t("dataGrid.rowActionsColumn")} />}
        </div>

        {/* Virtual rows */}
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
                  "grid absolute w-full border-b border-[var(--border-subtle)]/60 text-[12.5px] transition-colors" +
                  (isSelected ? " bg-primary/10" : " hover:bg-[var(--surface-hover)]")
                }
                style={{ ...gridStyle, top: virtualRow.start, height: "var(--grid-row-height)" }}
                data-index={virtualRow.index}
                role="row"
                aria-rowindex={virtualRow.index + 1}
                aria-selected={isSelected}
              >
                {/* Row number — clickable for selection (B1.3) */}
                <div
                  className={
                    "flex cursor-pointer select-none items-center border-r border-[var(--border-subtle)] px-2 text-[11px] tabular-nums" +
                    (isSelected
                      ? " bg-primary/20 font-medium text-primary"
                      : " bg-[var(--surface-editor)] text-[var(--text-tertiary)]")
                  }
                  style={{ position: "sticky", left: 0, zIndex: 1 }}
                  onClick={(e) => handleRowNumberClick(e, virtualRow.index)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter" || e.key === " ") {
                      e.preventDefault();
                      handleRowNumberClick(e, virtualRow.index);
                    }
                  }}
                  role="rowheader"
                  aria-label={t("dataGrid.rowNumber", { row: virtualRow.index + 1 })}
                >
                  {virtualRow.index + 1}
                </div>

                {/* Cells */}
                {orderedColumns.map((col, colIndex) => {
                  const colIdx = getColumnIndex(col);
                  const cell = row[colIdx];
                  const display = renderCellValue(cell);
                  const isNull = cell.type === "null";
                  const isJson = cell.type === "json";
                  const isEditing =
                    editingCell?.row === virtualRow.index && editingCell?.col === colIdx;
                  const isFrozen = frozenSet.has(col.name);
                  const frozenOffset = isFrozen ? stickyOffsets.get(col.name) : undefined;
                  const isLastFrozen = isFrozen && colIndex === frozenCols.length - 1;

                  return (
                    <div
                      key={col.name}
                      className={
                        "relative flex items-center overflow-hidden whitespace-nowrap px-3" +
                        (isNull ? " text-[var(--text-tertiary)]" : " text-foreground") +
                        (isFrozen && !isSelected
                          ? " bg-[var(--surface-editor)]"
                          : isFrozen
                            ? " bg-primary/15"
                            : "") +
                        (isLastFrozen ? " border-r border-[var(--border-default)]" : "") +
                        cellAlignmentClass(cell)
                      }
                      style={
                        frozenOffset === undefined
                          ? undefined
                          : { position: "sticky", left: frozenOffset, zIndex: 1 }
                      }
                      role="gridcell"
                      title={isJson ? undefined : display}
                      onDoubleClick={() => handleDoubleClick(virtualRow.index, colIdx)}
                      onKeyDown={(e) => {
                        if (e.key !== "Enter" && e.key !== "F2") return;
                        e.preventDefault();
                        handleDoubleClick(virtualRow.index, colIdx);
                      }}
                      tabIndex={
                        canEditRows &&
                        onEditCell &&
                        isCellTypeEditable(normalizeColumnType(col.dataType))
                          ? 0
                          : undefined
                      }
                      aria-label={
                        canEditRows &&
                        onEditCell &&
                        isCellTypeEditable(normalizeColumnType(col.dataType))
                          ? t("dataGrid.editCell", {
                              row: virtualRow.index + 1,
                              column: col.name,
                            })
                          : undefined
                      }
                      onContextMenu={(e) =>
                        handleCellContextMenu(e, col.name, virtualRow.index, colIdx)
                      }
                    >
                      {isEditing && onCellSave && onEditCell ? (
                        renderCellEditor ? (
                          renderCellEditor(cell, col.name)
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
                        <CellValueView cell={cell} display={display} />
                      )}
                    </div>
                  );
                })}

                {/* Row actions */}
                {hasRowActions && (
                  <div className="flex items-center justify-center gap-0.5 px-1" role="gridcell">
                    {onEditRow && (
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <Button
                            type="button"
                            variant="ghost"
                            size="sm"
                            className="h-6 w-6 p-0 text-[var(--text-secondary)] hover:bg-primary/10 hover:text-primary"
                            onClick={() => onEditRow(virtualRow.index)}
                            aria-label={t("dataGrid.editRow", { row: virtualRow.index + 1 })}
                          >
                            <Pencil className="h-3.5 w-3.5" />
                          </Button>
                        </TooltipTrigger>
                        <TooltipContent>
                          {t("dataGrid.editRow", { row: virtualRow.index + 1 })}
                        </TooltipContent>
                      </Tooltip>
                    )}
                    {onDeleteRow && (
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <Button
                            type="button"
                            variant="ghost"
                            size="sm"
                            className="h-6 w-6 p-0 text-[var(--text-secondary)] hover:bg-destructive/10 hover:text-destructive"
                            disabled={isDeleting}
                            onClick={() => onDeleteRow(virtualRow.index)}
                            aria-label={t("dataGrid.deleteRow", { row: virtualRow.index + 1 })}
                          >
                            <Trash2 className="h-3.5 w-3.5" />
                          </Button>
                        </TooltipTrigger>
                        <TooltipContent>
                          {t("dataGrid.deleteRow", { row: virtualRow.index + 1 })}
                        </TooltipContent>
                      </Tooltip>
                    )}
                  </div>
                )}
              </div>
            );
          })}
        </div>
      </div>

      {/* Footer */}
      {footer && (
        <div className="flex items-center gap-4 border-t border-[var(--border-subtle)] bg-muted/20 px-3 py-1.5 text-[11px] text-[var(--text-secondary)]">
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
  const initialText =
    value.type === "null" ? "" : String((value as { value: unknown }).value ?? "");
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
      className="absolute inset-0 z-10 h-full w-full border border-primary bg-background px-2 text-xs outline-none"
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
        className="cursor-text text-left text-[var(--text-secondary)] hover:text-foreground"
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
        className="cursor-text whitespace-pre-wrap text-left text-[var(--text-secondary)] hover:text-foreground"
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
    <span className="text-[var(--text-secondary)]" title={text}>
      {text}
    </span>
  );
}
