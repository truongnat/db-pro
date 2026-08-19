import { UnifiedGrid } from "@/modules/unified-grid/components/unified-grid";
import type { CellValue } from "@/modules/unified-grid/types";
import { useTranslation } from "@/commons/locales/useTranslation";

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
  onEditRow?: (rowIdx: number) => void;
  isDeleting: boolean;
  isLoading: boolean;
  pkColumns: string[];
  isReadonlyConnection?: boolean;
  frozenColumns?: string[];
  hiddenColumns?: string[];
  onToggleFreezeColumn?: (column: string) => void;
  columnWidths?: Record<string, number>;
  onColumnWidthsChange?: (widths: Record<string, number>) => void;
  onKeyDown?: (e: React.KeyboardEvent) => void;
  selectedRows?: Set<number>;
  onSelectionChange?: (selected: Set<number>) => void;
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
  onEditRow,
  isDeleting,
  isLoading,
  pkColumns,
  isReadonlyConnection = false,
  frozenColumns,
  hiddenColumns,
  onToggleFreezeColumn,
  columnWidths,
  onColumnWidthsChange,
  onKeyDown,
  selectedRows,
  onSelectionChange,
}: DataGridProps) {
  const { t } = useTranslation();
  const canEdit = pkColumns.length > 0 && !isReadonlyConnection;

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      {isReadonlyConnection ? (
        <div className="flex items-center gap-1.5 border-b border-[var(--border-subtle)] bg-[var(--surface-panel)] px-3 py-1 text-[11px] text-[var(--text-secondary)]">
          <span className="text-[var(--state-warning)]">●</span>
          {t("dataGrid.readOnlyConnection")}
        </div>
      ) : !canEdit ? (
        <div className="flex items-center gap-1.5 border-b border-[var(--border-subtle)] bg-[var(--surface-panel)] px-3 py-1 text-[11px] text-[var(--text-secondary)]">
          <span className="text-[var(--state-warning)]">●</span>
          {t("dataGrid.readOnlyNoPk")}
        </div>
      ) : null}
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
        onEditRow={onEditRow}
        isDeleting={isDeleting}
        isLoading={isLoading}
        frozenColumns={frozenColumns}
        hiddenColumns={hiddenColumns}
        onToggleFreezeColumn={onToggleFreezeColumn}
        columnWidths={columnWidths}
        onColumnWidthsChange={onColumnWidthsChange}
        selectedRows={selectedRows}
        onSelectionChange={onSelectionChange}
        emptyState={<EmptyState />}
        onKeyDown={onKeyDown}
        renderCellEditor={(cell, colName) => (
          <CellEditor
            value={cell}
            columnType={columns.find((c) => c.name === colName)?.dataType}
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
    </div>
  );
}
