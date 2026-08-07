import { UnifiedGrid } from "@/modules/unified-grid/components/unified-grid";
import type { CellValue } from "@/modules/unified-grid/types";

import { CellEditor } from "./cell-editor";
import { JsonCellRenderer } from "./json-cell-renderer";
import { EmptyState } from "./empty-state";

import type { ColumnMeta, GridSort, Row } from "../types/data-grid.types";

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
  hiddenColumns?: string[];
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
  frozenColumns,
  hiddenColumns,
  onToggleFreezeColumn,
}: DataGridProps) {
  const canEdit = pkColumns.length > 0;

  return (
    <UnifiedGrid
      columns={columns}
      rows={rows}
      sorts={sorts}
      onSort={onSort}
      editingCell={editingCell}
      onEditCell={onEditCell}
      onCellSave={onCellSave}
      canEditRows={canEdit}
      onDeleteRow={onDeleteRow}
      isDeleting={isDeleting}
      isLoading={isLoading}
      frozenColumns={frozenColumns}
      hiddenColumns={hiddenColumns}
      onToggleFreezeColumn={onToggleFreezeColumn}
      emptyState={<EmptyState />}
      renderCellEditor={(cell) => (
        <CellEditor
          value={cell}
          onSave={(newValue) => {
            if (editingCell) {
              onCellSave(editingCell.row, editingCell.col, newValue);
              onEditCell(null);
            }
          }}
          onCancel={() => onEditCell(null)}
        />
      )}
      renderJsonCell={(value) => <JsonCellRenderer value={value} />}
    />
  );
}
