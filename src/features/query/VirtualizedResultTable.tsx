import {
  type KeyboardEvent as ReactKeyboardEvent,
  type MouseEvent as ReactMouseEvent,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";

import {
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { cn } from "@/lib/utils";
import { copyTextToClipboard } from "./clipboard";
import {
  type CellPosition,
  type CellRange,
  type GridCopyFormat,
  type GridSelectionSummary,
  buildSelectionCopyPayload,
  isCellInsideRange,
  moveCellPosition,
  summarizeSelection,
} from "./grid-selection";

type VirtualizedResultTableProps = {
  columns: string[];
  rows: string[][];
  onCopyFeedback?: (message: string) => void;
  onSelectionSummaryChange?: (summary: GridSelectionSummary | null) => void;
};

const ROW_HEIGHT_PX = 34;
const OVERSCAN_ROWS = 12;
const FALLBACK_VIEWPORT_ROWS = 24;
const VIRTUALIZATION_THRESHOLD = 200;

export function VirtualizedResultTable({
  columns,
  rows,
  onCopyFeedback,
  onSelectionSummaryChange,
}: VirtualizedResultTableProps) {
  const containerRef = useRef<HTMLDivElement | null>(null);

  const [activeCell, setActiveCell] = useState<CellPosition | null>(null);
  const [selectionRange, setSelectionRange] = useState<CellRange | null>(null);

  const [scrollTop, setScrollTop] = useState(0);
  const [viewportHeight, setViewportHeight] = useState(0);

  const useVirtualizedRows = rows.length >= VIRTUALIZATION_THRESHOLD;

  useEffect(() => {
    setActiveCell(null);
    setSelectionRange(null);
    setScrollTop(0);
    onSelectionSummaryChange?.(null);
  }, [columns, onSelectionSummaryChange, rows]);

  useEffect(() => {
    const container = containerRef.current;
    if (!container || !useVirtualizedRows) {
      return;
    }

    const updateViewportHeight = () => {
      setViewportHeight(container.clientHeight);
    };

    updateViewportHeight();

    const resizeObserver = new ResizeObserver(updateViewportHeight);
    resizeObserver.observe(container);

    return () => {
      resizeObserver.disconnect();
    };
  }, [useVirtualizedRows]);

  const selectionSummary = useMemo(
    () => summarizeSelection(selectionRange),
    [selectionRange],
  );

  useEffect(() => {
    onSelectionSummaryChange?.(selectionSummary);
  }, [onSelectionSummaryChange, selectionSummary]);

  const selectSingleCell = useCallback((cell: CellPosition) => {
    setActiveCell(cell);
    setSelectionRange({ anchor: cell, focus: cell });
  }, []);

  const extendSelection = useCallback(
    (cell: CellPosition) => {
      setActiveCell(cell);
      setSelectionRange((previous) => {
        const anchor = previous?.anchor ?? activeCell ?? cell;
        return { anchor, focus: cell };
      });
    },
    [activeCell],
  );

  const isRangeSelected = Boolean(selectionRange && selectionSummary?.cellCount);

  const handleCopy = useCallback(
    async (format: GridCopyFormat, includeHeader: boolean) => {
      let payload = "";
      let message = "";

      if (selectionRange) {
        payload = buildSelectionCopyPayload(
          columns,
          rows,
          selectionRange,
          format,
          includeHeader,
        );

        const summary = summarizeSelection(selectionRange);
        if (summary) {
          message = `Copied ${summary.cellCount} cell(s) as ${format.toUpperCase()}.`;
        }
      } else if (rows.length > 0 && columns.length > 0) {
        const fullRange: CellRange = {
          anchor: { row: 0, column: 0 },
          focus: { row: rows.length - 1, column: columns.length - 1 },
        };
        payload = buildSelectionCopyPayload(
          columns,
          rows,
          fullRange,
          format,
          true,
        );
        message = `Copied ${rows.length} row(s) from current page as ${format.toUpperCase()}.`;
      }

      if (!payload.trim()) {
        return;
      }

      await copyTextToClipboard(payload);
      onCopyFeedback?.(message);
    },
    [columns, onCopyFeedback, rows, selectionRange],
  );

  const focusGridContainer = useCallback(() => {
    containerRef.current?.focus();
  }, []);

  const scrollCellIntoView = useCallback(
    (cell: CellPosition) => {
      const container = containerRef.current;
      if (!container) {
        return;
      }

      const selector = `[data-row-index="${cell.row}"][data-col-index="${cell.column}"]`;
      const target = container.querySelector<HTMLElement>(selector);
      if (target) {
        target.scrollIntoView({ block: "nearest", inline: "nearest" });
        return;
      }

      if (useVirtualizedRows) {
        const top = cell.row * ROW_HEIGHT_PX;
        const bottom = top + ROW_HEIGHT_PX;

        if (container.scrollTop > top) {
          container.scrollTop = top;
        } else if (container.scrollTop + container.clientHeight < bottom) {
          container.scrollTop = bottom - container.clientHeight;
        }
      }
    },
    [useVirtualizedRows],
  );

  const handleCellPointerSelect = useCallback(
    (
      rowIndex: number,
      columnIndex: number,
      shiftKey: boolean,
      event: ReactMouseEvent<HTMLTableCellElement>,
    ) => {
      if (event.button !== 0) {
        return;
      }

      event.preventDefault();
      focusGridContainer();

      const cell = { row: rowIndex, column: columnIndex };
      if (shiftKey) {
        extendSelection(cell);
      } else {
        selectSingleCell(cell);
      }
    },
    [extendSelection, focusGridContainer, selectSingleCell],
  );

  const handleKeyDown = useCallback(
    (event: ReactKeyboardEvent<HTMLDivElement>) => {
      const key = event.key;
      const withModifier = event.metaKey || event.ctrlKey;

      if (withModifier && key.toLowerCase() === "c") {
        event.preventDefault();
        const format: GridCopyFormat = event.shiftKey ? "csv" : "tsv";
        const includeHeader = format === "csv" || !isRangeSelected;
        void handleCopy(format, includeHeader).catch((error) => {
          onCopyFeedback?.(
            error instanceof Error ? error.message : "Failed to copy grid selection.",
          );
        });
        return;
      }

      if (!["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"].includes(key)) {
        return;
      }

      if (rows.length === 0 || columns.length === 0) {
        return;
      }

      event.preventDefault();

      if (!activeCell) {
        const firstCell = { row: 0, column: 0 };
        selectSingleCell(firstCell);
        scrollCellIntoView(firstCell);
        return;
      }

      const startCell = activeCell;
      const nextCell = moveCellPosition(startCell, key, rows.length, columns.length);
      if (!nextCell) {
        return;
      }

      setActiveCell(nextCell);
      setSelectionRange((previous) => {
        if (event.shiftKey) {
          const anchor = previous?.anchor ?? startCell;
          return { anchor, focus: nextCell };
        }
        return { anchor: nextCell, focus: nextCell };
      });
      scrollCellIntoView(nextCell);
    },
    [
      activeCell,
      columns.length,
      handleCopy,
      isRangeSelected,
      onCopyFeedback,
      rows.length,
      selectSingleCell,
      scrollCellIntoView,
    ],
  );

  const { startIndex, endIndex, topSpacerHeight, bottomSpacerHeight } = useMemo(() => {
    if (!useVirtualizedRows) {
      return {
        startIndex: 0,
        endIndex: rows.length,
        topSpacerHeight: 0,
        bottomSpacerHeight: 0,
      };
    }

    const viewportRows =
      viewportHeight > 0
        ? Math.ceil(viewportHeight / ROW_HEIGHT_PX)
        : FALLBACK_VIEWPORT_ROWS;

    const start = Math.max(Math.floor(scrollTop / ROW_HEIGHT_PX) - OVERSCAN_ROWS, 0);
    const end = Math.min(rows.length, start + viewportRows + OVERSCAN_ROWS * 2);

    return {
      startIndex: start,
      endIndex: end,
      topSpacerHeight: start * ROW_HEIGHT_PX,
      bottomSpacerHeight: Math.max((rows.length - end) * ROW_HEIGHT_PX, 0),
    };
  }, [rows.length, scrollTop, useVirtualizedRows, viewportHeight]);

  const visibleRows = useMemo(
    () => rows.slice(startIndex, endIndex),
    [endIndex, rows, startIndex],
  );

  const renderCell = useCallback(
    (value: string, rowIndex: number, colIndex: number) => {
      const inSelection = isCellInsideRange(rowIndex, colIndex, selectionRange);
      const isActiveCell =
        activeCell?.row === rowIndex && activeCell.column === colIndex;

      return (
        <TableCell
          key={`cell-${rowIndex}-${colIndex}`}
          data-row-index={rowIndex}
          data-col-index={colIndex}
          className={cn(
            "cursor-default whitespace-nowrap font-mono text-xs",
            inSelection && "bg-primary/5",
            isActiveCell && "bg-primary/15 ring-1 ring-inset ring-primary/45",
          )}
          tabIndex={-1}
          onMouseDown={(event) =>
            handleCellPointerSelect(rowIndex, colIndex, event.shiftKey, event)
          }
        >
          {value}
        </TableCell>
      );
    },
    [activeCell, handleCellPointerSelect, selectionRange],
  );

  return (
    <div
      ref={containerRef}
      className="h-full w-full overflow-auto"
      tabIndex={0}
      role="grid"
      aria-rowcount={rows.length}
      aria-colcount={columns.length}
      onScroll={(event) => {
        if (useVirtualizedRows) {
          setScrollTop(event.currentTarget.scrollTop);
        }
      }}
      onKeyDown={handleKeyDown}
    >
      <table className="min-w-max caption-bottom text-sm">
        <TableHeader className="sticky top-0 z-10 bg-background">
          <TableRow>
            {columns.map((column) => (
              <TableHead key={column} className="whitespace-nowrap">
                {column}
              </TableHead>
            ))}
          </TableRow>
        </TableHeader>

        <TableBody>
          {topSpacerHeight > 0 && (
            <TableRow aria-hidden="true" className="hover:bg-transparent">
              <TableCell
                colSpan={columns.length}
                className="p-0"
                style={{ height: topSpacerHeight }}
              />
            </TableRow>
          )}

          {visibleRows.map((row, rowOffset) => {
            const rowIndex = startIndex + rowOffset;
            const rowHeightStyle = useVirtualizedRows ? { height: ROW_HEIGHT_PX } : undefined;

            return (
              <TableRow key={`row-${rowIndex}`} style={rowHeightStyle}>
                {row.map((value, colIndex) => renderCell(value, rowIndex, colIndex))}
              </TableRow>
            );
          })}

          {bottomSpacerHeight > 0 && (
            <TableRow aria-hidden="true" className="hover:bg-transparent">
              <TableCell
                colSpan={columns.length}
                className="p-0"
                style={{ height: bottomSpacerHeight }}
              />
            </TableRow>
          )}
        </TableBody>
      </table>
    </div>
  );
}
