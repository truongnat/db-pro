import { useMemo } from "react";

import { useIntrospect } from "@/modules/schema/queries/schema.queries";
import { useTableRows, useUpdateRow, useDeleteRow } from "@/modules/data-grid/queries/data-grid.queries";
import { DataGrid } from "@/modules/data-grid/components/data-grid";
import { Pagination } from "@/modules/data-grid/components/pagination";
import { VisualFilterBuilder } from "@/modules/data-grid/components/visual-filter-builder";
import { useTabGridStateStore } from "@/modules/data-grid/state/tab-grid-state.store";
import type { CellValue, FetchRowsRequest, GridSort } from "@/modules/data-grid/types/data-grid.types";

import { ObjectSectionLayout } from "../object-section-layout";

interface DataSectionProps {
  tabId: string;
  connectionId: string;
  schema: string;
  table: string;
}

export function DataSection({ tabId, connectionId, schema, table }: DataSectionProps) {
  const tabState = useTabGridStateStore((s) => s.states[tabId]) ?? {
    filters: [], sorts: [] as GridSort[], page: 1, pageSize: 50,
    editingCell: null, frozenColumns: [] as string[], chartConfig: null,
  };
  const store = useTabGridStateStore.getState();

  const filters = tabState.filters;
  const sorts = tabState.sorts;
  const page = tabState.page;
  const pageSize = tabState.pageSize;
  const editingCell = tabState.editingCell;
  const frozenColumns = tabState.frozenColumns;

  const introspect = useIntrospect(connectionId);

  const pkColumns = useMemo(() => {
    if (!introspect.data) return [];
    return (
      introspect.data.primaryKeys
        .filter((pk) => pk.schema === schema && pk.tableName === table)
        .flatMap((pk) => pk.columns)
    );
  }, [introspect.data, schema, table]);

  const request: FetchRowsRequest = { schema, table, filters, sorts, page, pageSize };

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
    store.setSorts(tabId, newSorts);
  };

  const handleCellSave = (rowIdx: number, colIdx: number, value: CellValue) => {
    if (!pkColumns.length) return;
    const row = rows[rowIdx];
    const colNameToIdx = new Map(columns.map((c, i) => [c.name, i]));
    const updatedRow = [...row];
    updatedRow[colIdx] = value;
    const pkValues = pkColumns.map((pkCol) => {
      const idx = colNameToIdx.get(pkCol);
      return idx !== undefined ? row[idx] : { type: "null" as const };
    });
    updateRow.mutate({
      schema,
      table,
      columns: columns.map((c) => c.name),
      values: updatedRow,
      pkColumns,
      pkValues,
    });
  };

  const handleDeleteRow = (rowIdx: number) => {
    if (!pkColumns.length) return;
    const row = rows[rowIdx];
    const colNameToIdx = new Map(columns.map((c, i) => [c.name, i]));
    const pkValues = pkColumns.map((pkCol) => {
      const idx = colNameToIdx.get(pkCol);
      return idx !== undefined ? row[idx] : { type: "null" as const };
    });
    deleteRow.mutate({
      schema,
      table,
      columns: columns.map((c) => c.name),
      values: row,
      pkColumns,
      pkValues,
    });
  };

  return (
    <ObjectSectionLayout
      toolbar={
        columns.length > 0 ? (
          <VisualFilterBuilder
            columns={columns}
            filters={filters}
            onAddFilter={(f) => store.addFilter(tabId, f)}
            onRemoveFilter={(i) => store.removeFilter(tabId, i)}
          />
        ) : undefined
      }
      footer={
        <Pagination
          page={page}
          pageSize={pageSize}
          totalCount={totalCount}
          onPageChange={(p) => store.setPage(tabId, p)}
          onPageSizeChange={(s) => store.setPageSize(tabId, s)}
        />
      }
    >
      <DataGrid
        columns={columns}
        rows={rows}
        sorts={sorts}
        onSort={handleSort}
        editingCell={editingCell}
        onEditCell={(c) => store.setEditingCell(tabId, c)}
        onCellSave={handleCellSave}
        onDeleteRow={handleDeleteRow}
        isDeleting={deleteRow.isPending}
        isLoading={query.isFetching && !query.isPlaceholderData}
        pkColumns={pkColumns}
        frozenColumns={frozenColumns}
        onToggleFreezeColumn={(c) => store.toggleFrozenColumn(tabId, c)}
      />
    </ObjectSectionLayout>
  );
}
