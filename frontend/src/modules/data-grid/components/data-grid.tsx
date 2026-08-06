import { useRef, useCallback, useState } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";

import { renderCellValue } from "@/modules/query/types/query.types";

import { CellEditor } from "./cell-editor";
import { ColumnHeader } from "./column-header";
import { JsonCellRenderer } from "./json-cell-renderer";
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
  frozenColumns?: string[];
  onToggleFreezeColumn?: (column: string) => void;
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
  frozenColumns = [],
  onToggleFreezeColumn,
}: DataGridProps) {
  const parentRef = useRef<HTMLDivElement>(null);
  const canEdit = pkColumns.length > 0;
  const [contextMenu, setContextMenu] = useState<{ x: number; y: number; column: string } | null>(null);

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

  const handleContextMenu = useCallback(
    (e: React.MouseEvent, columnName: string) => {
      if (!onToggleFreezeColumn) return;
      e.preventDefault();
      setContextMenu({ x: e.clientX, y: e.clientY, column: columnName });
    },
    [onToggleFreezeColumn],
  );

  if (!columns.length) {
    return <EmptyState />;
  }

  const frozenSet = new Set(frozenColumns);
  const frozenCols = columns.filter((c) => frozenSet.has(c.name));
  const normalCols = columns.filter((c) => !frozenSet.has(c.name));
  const orderedCols = [...frozenCols, ...normalCols];

  const gridStyle: React.CSSProperties = {
    gridTemplateColumns: `40px repeat(${orderedCols.length}, minmax(120px, 1fr)) 40px`,
  };

  const sortMap = new Map(sorts.map((s) => [s.column, s]));

  const getColumnIndex = (col: ColumnMeta) => columns.findIndex((c) => c.name === col.name);

  return (
    <div className="relative flex h-full flex-col">
      {isLoading && <LoadingOverlay />}

      {contextMenu && (
        <div
          className="fixed z-50 rounded-[var(--radius-sm)] border py-1 shadow-lg"
          style={{
            left: contextMenu.x,
            top: contextMenu.y,
            backgroundColor: "var(--color-bg-secondary, #1e293b)",
            borderColor: "var(--color-border)",
          }}
          onMouseLeave={() => setContextMenu(null)}
        >
          <button
            type="button"
            className="block w-full px-3 py-1 text-left text-xs hover:bg-[var(--color-bg)]"
            style={{ color: "var(--color-text)" }}
            onClick={() => {
              onToggleFreezeColumn?.(contextMenu.column);
              setContextMenu(null);
            }}
          >
            {frozenSet.has(contextMenu.column) ? "Unfreeze" : "Freeze"} "{contextMenu.column}"
          </button>
        </div>
      )}

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
        {orderedCols.map((col) => (
          <div
            key={col.name}
            onContextMenu={(e) => handleContextMenu(e, col.name)}
            style={frozenSet.has(col.name) ? { backgroundColor: "var(--color-surface)", fontWeight: 600 } : undefined}
          >
            <ColumnHeader
              column={col}
              sort={sortMap.get(col.name)}
              onSort={onSort}
            />
          </div>
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
                {orderedCols.map((col) => {
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
                      className="relative overflow-hidden px-3 py-1.5 text-ellipsis whitespace-nowrap"
                      style={{
                        color: isNull
                          ? "var(--color-text-secondary)"
                          : "var(--color-text)",
                        fontStyle: isNull ? "italic" : undefined,
                        backgroundColor: isFrozen ? "var(--color-surface)" : undefined,
                      }}
                      title={isJson ? undefined : display}
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
                      ) : isJson && cell.type === "json" ? (
                        <JsonCellRenderer value={cell.value} />
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
