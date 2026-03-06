import { useCallback, useEffect, useMemo, useState } from "react";

const MIN_COLUMN_WIDTH_PX = 96;
const MAX_COLUMN_WIDTH_PX = 560;
const DEFAULT_COLUMN_WIDTH_PX = 180;

type UseColumnLayoutControllerOptions = {
  columns: string[];
  rows: string[][];
  onStatus: (message: string) => void;
};

export function useColumnLayoutController({
  columns,
  rows,
  onStatus,
}: UseColumnLayoutControllerOptions) {
  const [hiddenColumnSet, setHiddenColumnSet] = useState<Set<string>>(() => new Set());
  const [columnWidthByName, setColumnWidthByName] = useState<Record<string, number>>({});

  useEffect(() => {
    const validNames = new Set(columns);

    setHiddenColumnSet((previous) => {
      const next = new Set<string>();
      for (const name of previous) {
        if (validNames.has(name)) {
          next.add(name);
        }
      }
      return next;
    });

    setColumnWidthByName((previous) => {
      const next: Record<string, number> = {};
      for (const name of columns) {
        if (typeof previous[name] === "number") {
          next[name] = previous[name];
          continue;
        }
        next[name] = estimateColumnWidth(name, rows, columns.indexOf(name));
      }
      return next;
    });
  }, [columns, rows]);

  const visibleColumns = useMemo(
    () => columns.filter((name) => !hiddenColumnSet.has(name)),
    [columns, hiddenColumnSet],
  );

  const visibleColumnIndexes = useMemo(() => {
    const nameToIndex = new Map<string, number>();
    for (let index = 0; index < columns.length; index += 1) {
      nameToIndex.set(columns[index], index);
    }

    return visibleColumns
      .map((name) => nameToIndex.get(name))
      .filter((index): index is number => typeof index === "number" && index >= 0);
  }, [columns, visibleColumns]);

  const hiddenColumnCount = columns.length - visibleColumns.length;

  const canHideAnyColumn = visibleColumns.length > 1;

  const toggleColumnVisibility = useCallback(
    (columnName: string) => {
      setHiddenColumnSet((previous) => {
        const next = new Set(previous);
        const currentlyVisibleCount = columns.length - previous.size;
        if (next.has(columnName)) {
          next.delete(columnName);
          onStatus(`Column '${columnName}' is now visible.`);
          return next;
        }

        if (currentlyVisibleCount <= 1) {
          onStatus("At least one column must remain visible.");
          return previous;
        }

        next.add(columnName);
        onStatus(`Column '${columnName}' hidden.`);
        return next;
      });
    },
    [columns.length, onStatus],
  );

  const showAllColumns = useCallback(() => {
    setHiddenColumnSet(new Set());
    onStatus("All columns are visible.");
  }, [onStatus]);

  const setColumnWidth = useCallback(
    (columnName: string, nextWidth: number) => {
      const clampedWidth = clampColumnWidth(nextWidth);
      setColumnWidthByName((previous) => ({
        ...previous,
        [columnName]: clampedWidth,
      }));
    },
    [],
  );

  const resetColumnLayout = useCallback(() => {
    setHiddenColumnSet(new Set());
    setColumnWidthByName(() => {
      const next: Record<string, number> = {};
      for (let index = 0; index < columns.length; index += 1) {
        const name = columns[index];
        next[name] = estimateColumnWidth(name, rows, index);
      }
      return next;
    });
    onStatus("Column visibility and widths were reset.");
  }, [columns, onStatus, rows]);

  return {
    visibleColumns,
    visibleColumnIndexes,
    hiddenColumnSet,
    hiddenColumnCount,
    canHideAnyColumn,
    columnWidthByName,
    toggleColumnVisibility,
    showAllColumns,
    setColumnWidth,
    resetColumnLayout,
  };
}

export function clampColumnWidth(width: number): number {
  if (!Number.isFinite(width)) {
    return DEFAULT_COLUMN_WIDTH_PX;
  }
  return Math.max(MIN_COLUMN_WIDTH_PX, Math.min(MAX_COLUMN_WIDTH_PX, Math.round(width)));
}

function estimateColumnWidth(
  columnName: string,
  rows: string[][],
  columnIndex: number,
): number {
  let maxLength = Math.max(6, columnName.length);
  const sampleRows = Math.min(60, rows.length);

  for (let rowIndex = 0; rowIndex < sampleRows; rowIndex += 1) {
    const value = rows[rowIndex]?.[columnIndex] ?? "";
    maxLength = Math.max(maxLength, value.length);
  }

  const estimated = maxLength * 7.4 + 42;
  return clampColumnWidth(estimated);
}
