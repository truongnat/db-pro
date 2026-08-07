import { useEffect, useMemo } from "react";

import { useConnectionStore } from "@/commons/stores/connection.store";
import { useIntrospect } from "@/modules/schema/queries/schema.queries";

import { ChartView } from "./chart-view";
import { DataGrid } from "./data-grid";
import { Pagination } from "./pagination";
import { VisualFilterBuilder } from "./visual-filter-builder";
import { useDataGridModuleStore } from "../state/data-grid.store";
import {
  useTableRows,
  useUpdateRow,
  useDeleteRow,
} from "../queries/data-grid.queries";
import type { CellValue, FetchRowsRequest, GridSort } from "../types/data-grid.types";

interface TableDataTabContentProps {
  connectionId: string | null;
  schema: string;
  table: string;
}

export function TableDataTabContent({
  connectionId,
  schema,
  table,
}: TableDataTabContentProps) {
  const storeConnectionId = useDataGridModuleStore((s) => s.connectionId);
  const tableSchema = useDataGridModuleStore((s) => s.tableSchema);
  const tableName = useDataGridModuleStore((s) => s.tableName);
  const filters = useDataGridModuleStore((s) => s.filters);
  const sorts = useDataGridModuleStore((s) => s.sorts);
  const page = useDataGridModuleStore((s) => s.page);
  const pageSize = useDataGridModuleStore((s) => s.pageSize);
  const editingCell = useDataGridModuleStore((s) => s.editingCell);
  const frozenColumns = useDataGridModuleStore((s) => s.frozenColumns);
  const chartConfig = useDataGridModuleStore((s) => s.chartConfig);
  const setTable = useDataGridModuleStore((s) => s.setTable);
  const addFilter = useDataGridModuleStore((s) => s.addFilter);
  const removeFilter = useDataGridModuleStore((s) => s.removeFilter);
  const setSorts = useDataGridModuleStore((s) => s.setSorts);
  const setPage = useDataGridModuleStore((s) => s.setPage);
  const setPageSize = useDataGridModuleStore((s) => s.setPageSize);
  const setEditingCell = useDataGridModuleStore((s) => s.setEditingCell);
  const toggleFrozenColumn = useDataGridModuleStore((s) => s.toggleFrozenColumn);

  useEffect(() => {
    if (storeConnectionId !== connectionId) {
      useDataGridModuleStore.getState().reset();
      useDataGridModuleStore.setState({ connectionId });
    }
  }, [connectionId, storeConnectionId]);

  useEffect(() => {
    if (tableSchema !== schema || tableName !== table) {
      setTable(schema, table);
    }
  }, [schema, table, tableSchema, tableName, setTable]);

  const introspect = useIntrospect(connectionId);

  const pkColumns = useMemo(() => {
    if (!introspect.data) return [];
    return (
      introspect.data.primaryKeys
        .filter((pk) => pk.schema === schema && pk.tableName === table)
        .flatMap((pk) => pk.columns)
    );
  }, [introspect.data, schema, table]);

  const request: FetchRowsRequest | null =
    tableSchema && tableName
      ? { schema: tableSchema, table: tableName, filters, sorts, page, pageSize }
      : null;

  const query = useTableRows(connectionId, request);
  const updateRow = useUpdateRow(connectionId, request);
  const deleteRow = useDeleteRow(connectionId, request);

  const columns = query.data?.columns ?? [];
  const rows = query.data?.rows ?? [];
  const totalCount = query.data?.totalCount ?? 0;

  const handleSort = (column: string) => {
    const existing = sorts.find((s) => s.column === column);
    let newSorts: GridSort[];
    if (!existing) {
      newSorts = [{ column, direction: "asc" }];
    } else if (existing.direction === "asc") {
      newSorts = [{ column, direction: "desc" }];
    } else {
      newSorts = [];
    }
    setSorts(newSorts);
  };

  const handleCellSave = (rowIdx: number, colIdx: number, value: CellValue) => {
    if (!tableSchema || !tableName || !pkColumns.length) return;
    const row = rows[rowIdx];
    const col = columns[colIdx];
    const colNameToIdx = new Map(columns.map((c, i) => [c.name, i]));
    const updatedRow = [...row];
    updatedRow[colIdx] = value;
    const pkValues = pkColumns.map((pkCol) => {
      const idx = colNameToIdx.get(pkCol);
      return idx !== undefined ? row[idx] : { type: "null" as const };
    });
    updateRow.mutate({
      schema: tableSchema,
      table: tableName,
      columns: columns.map((c) => c.name),
      values: updatedRow,
      pkColumns,
      pkValues,
    });
  };

  const handleDeleteRow = (rowIdx: number) => {
    if (!tableSchema || !tableName || !pkColumns.length) return;
    const row = rows[rowIdx];
    const colNameToIdx = new Map(columns.map((c, i) => [c.name, i]));
    const pkValues = pkColumns.map((pkCol) => {
      const idx = colNameToIdx.get(pkCol);
      return idx !== undefined ? row[idx] : { type: "null" as const };
    });
    deleteRow.mutate({
      schema: tableSchema,
      table: tableName,
      columns: columns.map((c) => c.name),
      values: row,
      pkColumns,
      pkValues,
    });
  };

  return (
    <div className="flex h-full flex-col">
      {columns.length > 0 && (
        <VisualFilterBuilder
          columns={columns}
          filters={filters}
          onAddFilter={addFilter}
          onRemoveFilter={removeFilter}
        />
      )}

      <div className="relative flex-1 overflow-hidden">
        <DataGrid
          columns={columns}
          rows={rows}
          sorts={sorts}
          onSort={handleSort}
          editingCell={editingCell}
          onEditCell={setEditingCell}
          onCellSave={handleCellSave}
          onDeleteRow={handleDeleteRow}
          isDeleting={deleteRow.isPending}
          isLoading={query.isFetching && !query.isPlaceholderData}
          pkColumns={pkColumns}
          frozenColumns={frozenColumns}
          onToggleFreezeColumn={toggleFrozenColumn}
        />
      </div>

      <Pagination
        page={page}
        pageSize={pageSize}
        totalCount={totalCount}
        onPageChange={setPage}
        onPageSizeChange={setPageSize}
      />
    </div>
  );
}
