import { useEffect, useMemo } from "react";

import { useConnectionStore } from "@/commons/stores/connection.store";
import { useTranslation } from "@/commons/locales/useTranslation";
import { useIntrospect } from "@/modules/schema/queries/schema.queries";

import { DataGrid } from "../components/data-grid";
import { FilterBar } from "../components/filter-bar";
import { Pagination } from "../components/pagination";
import { EmptyState } from "../components/empty-state";
import { useDataGridModuleStore } from "../state/data-grid.store";
import {
  useTableRows,
  useUpdateRow,
  useDeleteRow,
} from "../queries/data-grid.queries";
import type { CellValue, FetchRowsRequest, GridSort } from "../types/data-grid.types";

export function DataPage() {
  const { t } = useTranslation();
  const activeConnectionId = useConnectionStore((s) => s.activeConnectionId);

  const storeConnectionId = useDataGridModuleStore((s) => s.connectionId);
  const tableSchema = useDataGridModuleStore((s) => s.tableSchema);
  const tableName = useDataGridModuleStore((s) => s.tableName);
  const filters = useDataGridModuleStore((s) => s.filters);
  const sorts = useDataGridModuleStore((s) => s.sorts);
  const page = useDataGridModuleStore((s) => s.page);
  const pageSize = useDataGridModuleStore((s) => s.pageSize);
  const editingCell = useDataGridModuleStore((s) => s.editingCell);
  const setTable = useDataGridModuleStore((s) => s.setTable);
  const addFilter = useDataGridModuleStore((s) => s.addFilter);
  const removeFilter = useDataGridModuleStore((s) => s.removeFilter);
  const setSorts = useDataGridModuleStore((s) => s.setSorts);
  const setPage = useDataGridModuleStore((s) => s.setPage);
  const setPageSize = useDataGridModuleStore((s) => s.setPageSize);
  const setEditingCell = useDataGridModuleStore((s) => s.setEditingCell);

  useEffect(() => {
    if (storeConnectionId !== activeConnectionId) {
      useDataGridModuleStore.getState().reset();
      useDataGridModuleStore.setState({ connectionId: activeConnectionId });
    }
  }, [activeConnectionId, storeConnectionId]);

  const introspect = useIntrospect(activeConnectionId);

  const tableOptions = useMemo(() => {
    if (!introspect.data) return [];
    return introspect.data.tables.map((t) => ({ name: t.name, schema: t.schema }));
  }, [introspect.data]);

  const pkColumns = useMemo(() => {
    if (!introspect.data || !tableSchema || !tableName) return [];
    return (
      introspect.data.primaryKeys
        .filter((pk) => pk.schema === tableSchema && pk.tableName === tableName)
        .flatMap((pk) => pk.columns)
    );
  }, [introspect.data, tableSchema, tableName]);

  const request: FetchRowsRequest | null =
    tableSchema && tableName
      ? { schema: tableSchema, table: tableName, filters, sorts, page, pageSize }
      : null;

  const query = useTableRows(activeConnectionId, request);
  const updateRow = useUpdateRow(activeConnectionId, request);
  const deleteRow = useDeleteRow(activeConnectionId, request);

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

    const updateColumns = columns.map((c) => c.name);
    const updateValues = updatedRow;

    const pkValues = pkColumns.map((pkCol) => {
      const idx = colNameToIdx.get(pkCol);
      return idx !== undefined ? row[idx] : { type: "null" as const };
    });

    updateRow.mutate({
      schema: tableSchema,
      table: tableName,
      columns: updateColumns,
      values: updateValues,
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

  if (!activeConnectionId) {
    return (
      <div className="flex flex-1 items-center justify-center text-sm" style={{ color: "var(--color-text-secondary)" }}>
        {t("dataGrid.connectFirst")}
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col">
      <div
        className="flex items-center gap-2 border-b px-3 py-2"
        style={{ borderColor: "var(--color-border)", backgroundColor: "var(--color-surface)" }}
      >
        <label className="text-xs font-medium" style={{ color: "var(--color-text-secondary)" }}>
          {t("dataGrid.selectTable")}
        </label>
        <select
          className="rounded border px-2 py-1 text-sm"
          style={{
            borderColor: "var(--color-border)",
            backgroundColor: "var(--color-surface)",
            color: "var(--color-text)",
          }}
          value={tableName ? `${tableSchema}.${tableName}` : ""}
          onChange={(e) => {
            const val = e.target.value;
            if (!val) {
              setTable(null, null);
            } else {
              const [schema, table] = val.split(".");
              setTable(schema, table);
            }
          }}
        >
          <option value="">{t("dataGrid.selectTable")}</option>
          {tableOptions.map((opt) => (
            <option key={`${opt.schema}.${opt.name}`} value={`${opt.schema}.${opt.name}`}>
              {opt.schema}.{opt.name}
            </option>
          ))}
        </select>
      </div>

      {!tableName ? (
        <EmptyState message={t("dataGrid.selectTable")} />
      ) : (
        <>
          {columns.length > 0 && (
            <FilterBar
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
            />
          </div>

          <Pagination
            page={page}
            pageSize={pageSize}
            totalCount={totalCount}
            onPageChange={setPage}
            onPageSizeChange={setPageSize}
          />
        </>
      )}
    </div>
  );
}
