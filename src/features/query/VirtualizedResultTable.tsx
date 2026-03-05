import { type KeyboardEvent as ReactKeyboardEvent, useEffect, useMemo, useRef, useState } from "react";

import {
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { cn } from "@/lib/utils";
import { copyTextToClipboard, toTabDelimited } from "./clipboard";

type VirtualizedResultTableProps = {
  columns: string[];
  rows: string[][];
  onCopyFeedback?: (message: string) => void;
};

const ROW_HEIGHT_PX = 34;
const OVERSCAN_ROWS = 12;
const FALLBACK_VIEWPORT_ROWS = 24;
const VIRTUALIZATION_THRESHOLD = 200;

type CellPosition = {
  row: number;
  column: number;
};

function copySelectionToText(
  columns: string[],
  rows: string[][],
  selectedCell: CellPosition | null,
): { text: string; message: string } {
  if (selectedCell) {
    const value = rows[selectedCell.row]?.[selectedCell.column];
    if (typeof value === "string") {
      return {
        text: value,
        message: `Copied cell [${selectedCell.row + 1}, ${selectedCell.column + 1}].`,
      };
    }
  }

  const text = toTabDelimited(columns, rows, true);
  return {
    text,
    message: `Copied ${rows.length} row(s) from current page.`,
  };
}

export function VirtualizedResultTable({
  columns,
  rows,
  onCopyFeedback,
}: VirtualizedResultTableProps) {
  const [selectedCell, setSelectedCell] = useState<CellPosition | null>(null);

  useEffect(() => {
    setSelectedCell(null);
  }, [columns, rows]);

  const handleCopy = async () => {
    const selection = copySelectionToText(columns, rows, selectedCell);
    if (!selection.text.trim()) {
      return;
    }
    await copyTextToClipboard(selection.text);
    onCopyFeedback?.(selection.message);
  };

  const handleKeyDown = (event: ReactKeyboardEvent<HTMLDivElement>) => {
    if (!(event.metaKey || event.ctrlKey) || event.key.toLowerCase() !== "c") {
      return;
    }

    event.preventDefault();
    void handleCopy().catch((error) => {
      onCopyFeedback?.(
        error instanceof Error ? error.message : "Failed to copy grid selection.",
      );
    });
  };

  if (rows.length < VIRTUALIZATION_THRESHOLD) {
    return (
      <div className="h-full w-full overflow-auto" tabIndex={0} onKeyDown={handleKeyDown}>
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
            {rows.map((row, rowIndex) => (
              <TableRow key={`row-${rowIndex}`}>
                {row.map((value, colIndex) => (
                  <TableCell
                    key={`cell-${rowIndex}-${colIndex}`}
                    className={cn(
                      "whitespace-nowrap font-mono text-xs",
                      selectedCell?.row === rowIndex &&
                        selectedCell.column === colIndex &&
                        "bg-primary/10 ring-1 ring-inset ring-primary/40",
                    )}
                    tabIndex={0}
                    onClick={() =>
                      setSelectedCell({ row: rowIndex, column: colIndex })
                    }
                    onFocus={() =>
                      setSelectedCell({ row: rowIndex, column: colIndex })
                    }
                  >
                    {value}
                  </TableCell>
                ))}
              </TableRow>
            ))}
          </TableBody>
        </table>
      </div>
    );
  }

  return (
    <VirtualizedTable
      columns={columns}
      rows={rows}
      selectedCell={selectedCell}
      onSelectedCellChange={setSelectedCell}
      onKeyDown={handleKeyDown}
    />
  );
}

type VirtualizedTableProps = {
  columns: string[];
  rows: string[][];
  selectedCell: CellPosition | null;
  onSelectedCellChange: (cell: CellPosition) => void;
  onKeyDown: (event: ReactKeyboardEvent<HTMLDivElement>) => void;
};

function VirtualizedTable({
  columns,
  rows,
  selectedCell,
  onSelectedCellChange,
  onKeyDown,
}: VirtualizedTableProps) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const [scrollTop, setScrollTop] = useState(0);
  const [viewportHeight, setViewportHeight] = useState(0);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) {
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
  }, []);

  const { startIndex, endIndex, topSpacerHeight, bottomSpacerHeight } = useMemo(() => {
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
  }, [rows.length, scrollTop, viewportHeight]);

  const visibleRows = useMemo(
    () => rows.slice(startIndex, endIndex),
    [endIndex, rows, startIndex],
  );

  return (
    <div
      ref={containerRef}
      className="h-full w-full overflow-auto"
      tabIndex={0}
      onScroll={(event) => setScrollTop(event.currentTarget.scrollTop)}
      onKeyDown={onKeyDown}
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
              <TableCell colSpan={columns.length} className="p-0" style={{ height: topSpacerHeight }} />
            </TableRow>
          )}

          {visibleRows.map((row, rowOffset) => {
            const rowIndex = startIndex + rowOffset;
            return (
              <TableRow key={`row-${rowIndex}`} style={{ height: ROW_HEIGHT_PX }}>
                {row.map((value, colIndex) => (
                  <TableCell
                    key={`cell-${rowIndex}-${colIndex}`}
                    className={cn(
                      "whitespace-nowrap font-mono text-xs",
                      selectedCell?.row === rowIndex &&
                        selectedCell.column === colIndex &&
                        "bg-primary/10 ring-1 ring-inset ring-primary/40",
                    )}
                    tabIndex={0}
                    onClick={() =>
                      onSelectedCellChange({ row: rowIndex, column: colIndex })
                    }
                    onFocus={() =>
                      onSelectedCellChange({ row: rowIndex, column: colIndex })
                    }
                  >
                    {value}
                  </TableCell>
                ))}
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
