import { toTabDelimited } from "./clipboard";
import { toCsv } from "./csv";

export type CellPosition = {
  row: number;
  column: number;
};

export type CellRange = {
  anchor: CellPosition;
  focus: CellPosition;
};

export type GridCopyFormat = "tsv" | "csv";

export type GridSelectionSummary = {
  rowCount: number;
  columnCount: number;
  cellCount: number;
};

type NormalizedCellRange = {
  top: number;
  bottom: number;
  left: number;
  right: number;
};

export function clampCellPosition(
  cell: CellPosition,
  maxRows: number,
  maxColumns: number,
): CellPosition {
  return {
    row: clampIndex(cell.row, maxRows),
    column: clampIndex(cell.column, maxColumns),
  };
}

export function normalizeCellRange(range: CellRange): NormalizedCellRange {
  const top = Math.min(range.anchor.row, range.focus.row);
  const bottom = Math.max(range.anchor.row, range.focus.row);
  const left = Math.min(range.anchor.column, range.focus.column);
  const right = Math.max(range.anchor.column, range.focus.column);

  return { top, bottom, left, right };
}

export function isCellInsideRange(
  row: number,
  column: number,
  range: CellRange | null,
): boolean {
  if (!range) {
    return false;
  }

  const normalized = normalizeCellRange(range);
  return (
    row >= normalized.top &&
    row <= normalized.bottom &&
    column >= normalized.left &&
    column <= normalized.right
  );
}

export function summarizeSelection(range: CellRange | null): GridSelectionSummary | null {
  if (!range) {
    return null;
  }

  const normalized = normalizeCellRange(range);
  const rowCount = normalized.bottom - normalized.top + 1;
  const columnCount = normalized.right - normalized.left + 1;

  return {
    rowCount,
    columnCount,
    cellCount: rowCount * columnCount,
  };
}

export function moveCellPosition(
  current: CellPosition,
  key: string,
  maxRows: number,
  maxColumns: number,
): CellPosition | null {
  switch (key) {
    case "ArrowUp":
      return clampCellPosition({ row: current.row - 1, column: current.column }, maxRows, maxColumns);
    case "ArrowDown":
      return clampCellPosition({ row: current.row + 1, column: current.column }, maxRows, maxColumns);
    case "ArrowLeft":
      return clampCellPosition({ row: current.row, column: current.column - 1 }, maxRows, maxColumns);
    case "ArrowRight":
      return clampCellPosition({ row: current.row, column: current.column + 1 }, maxRows, maxColumns);
    default:
      return null;
  }
}

export function buildSelectionCopyPayload(
  columns: string[],
  rows: string[][],
  range: CellRange,
  format: GridCopyFormat,
  includeHeader: boolean,
): string {
  const normalized = normalizeCellRange(range);
  const selectedColumns = columns.slice(normalized.left, normalized.right + 1);
  const selectedRows = rows
    .slice(normalized.top, normalized.bottom + 1)
    .map((row) => row.slice(normalized.left, normalized.right + 1));

  if (format === "csv") {
    if (includeHeader) {
      return toCsv(selectedColumns, selectedRows);
    }

    return selectedRows.map(serializeCsvRow).join("\r\n");
  }

  return toTabDelimited(selectedColumns, selectedRows, includeHeader);
}

function serializeCsvRow(row: string[]): string {
  return row
    .map((value) => {
      if (/[",\r\n]/.test(value)) {
        return `"${value.replace(/"/g, "\"\"")}"`;
      }
      return value;
    })
    .join(",");
}

function clampIndex(value: number, max: number): number {
  if (max <= 0) {
    return 0;
  }

  if (value < 0) {
    return 0;
  }
  if (value >= max) {
    return max - 1;
  }
  return value;
}
