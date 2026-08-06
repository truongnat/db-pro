import { useRef, useCallback } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";

import { renderCellValue } from "@/modules/query/types/query.types";

import { CellEditor } from "./cell-editor";
import { ColumnHeader } from "./column-header";
import { RowActions } from "./row-actions";
import { EmptyState } from "./empty-state";
import { LoadingOverlay } from "./loading-overlay";

import type { CellValue, ColumnMeta, GridSort, Row } from "../types/data-grid.types";

interface DataGridProps {
  columns: ColumnMeta[];
  rows: Row[];
  sorts: GridSort[];
  onSort: (column: string) => void;
  editingCell: { row: number; col: number } | null;
  onEditCell: (cell: { row: number; col: number } | null) => void;
  onCellSave: (rowIdx: number, colIdx: number, value: CellValue) => void;
  onDeleteRow: (rowIdx: number) => void;
  isDeleting: boolean;
  isLoading: boolean;
  pkColumns: string[];
}

export function DataGrid({
  columns,
  rows,
  sorts,
  onSort,
  editingCell,
  onEditCell,
  onCellSave,
  onDeleteRow,
  isDeleting,
  isLoading,
  pkColumns,
}: DataGridProps) {
  const parentRef = useRef<HTMLDivElement>(null);
  const canEdit = pkColumns.length > 0;

  const virtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 32,
    overscan: 10,
  });

  const handleDoubleClick = useCallback(
    (rowIdx: number, colIdx: number) => {
      if (!canEdit) return;
      onEditCell({ row: rowIdx, col: colIdx });
    },
    [canEdit, onEditCell],
  );

  if (!columns.length) {
    return <EmptyState />;
  }

  const gridStyle: React.CSSProperties = {
    gridTemplateColumns: `40px repeat(${columns.length}, minmax(120px, 1fr)) 40px`,
  };

  const sortMap = new Map(sorts.map((s) => [s.column, s]));

  return (
    <div className="relative flex h-full flex-col">
      {isLoading && <LoadingOverlay />}

      <div
        className="grid shrink-0 border-b text-xs font-medium"
        style={{
          ...gridStyle,
          backgroundColor: "var(--color-surface)",
          borderColor: "var(--color-border)",
        }}
      >
        <div className="px-2 py-2" style={{ color: "var(--color-text-secondary)" }}>
          #
        </div>
        {columns.map((col) => (
          <ColumnHeader
            key={col.name}
            column={col}
            sort={sortMap.get(col.name)}
            onSort={onSort}
          />
        ))}
        <div />
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
                  const isEditing =
                    editingCell?.row === virtualRow.index &&
                    editingCell?.col === colIdx;

                  return (
                    <div
                      key={colIdx}
                      className="relative overflow-hidden px-3 py-1.5 text-ellipsis whitespace-nowrap"
                      style={{
                        color: isNull
                          ? "var(--color-text-secondary)"
                          : "var(--color-text)",
                        fontStyle: isNull ? "italic" : undefined,
                      }}
                      title={display}
                      onDoubleClick={() => handleDoubleClick(virtualRow.index, colIdx)}
                    >
                      {isEditing ? (
                        <CellEditor
                          value={cell}
                          onSave={(newValue) => {
                            onCellSave(virtualRow.index, colIdx, newValue);
                            onEditCell(null);
                          }}
                          onCancel={() => onEditCell(null)}
                        />
                      ) : (
                        display
                      )}
                    </div>
                  );
                })}
                <div className="flex items-center justify-center px-1">
                  {canEdit && (
                    <RowActions
                      onDelete={() => onDeleteRow(virtualRow.index)}
                      isDeleting={isDeleting}
                    />
                  )}
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
