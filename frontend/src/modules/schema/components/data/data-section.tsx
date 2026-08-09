import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { useTranslation } from "@/commons/locales/useTranslation";

import { useIntrospect } from "@/modules/schema/queries/schema.queries";
import { useTableRows, useUpdateRow, useDeleteRow } from "@/modules/data-grid/queries/data-grid.queries";
import { DataGrid } from "@/modules/data-grid/components/data-grid";
import { ChangeBar } from "@/modules/data-grid/components/change-bar";
import { Pagination } from "@/modules/data-grid/components/pagination";
import { DataToolbar } from "@/modules/data-grid/components/data-toolbar";
import { useTabGridStateStore } from "@/modules/data-grid/state/tab-grid-state.store";
import { useStagedChangesStore, useTabStagedChanges } from "@/modules/data-grid/state/staged-changes.store";
import { cycleColumnSort } from "@/modules/data-grid/utils/sort";
import type { CellValue, FetchRowsRequest, GridSort } from "@/modules/data-grid/types/data-grid.types";

import { ObjectSectionLayout } from "../object-section-layout";

interface DataSectionProps {
  tabId: string;
  connectionId: string;
  schema: string;
  table: string;
  /** Incremented by the parent when the header Refresh is clicked. */
  refreshCounter?: number;
}

export function DataSection({ tabId, connectionId, schema, table, refreshCounter = 0 }: DataSectionProps) {
  const { t } = useTranslation();
  const tabState = useTabGridStateStore((s) => s.states[tabId]) ?? {
    filters: [], sorts: [] as GridSort[], page: 1, pageSize: 50,
    editingCell: null, frozenColumns: [] as string[], hiddenColumns: [] as string[], chartConfig: null,
  };
  const store = useTabGridStateStore.getState();

  const filters = tabState.filters;
  const sorts = tabState.sorts;
  const page = tabState.page;
  const pageSize = tabState.pageSize;
  const editingCell = tabState.editingCell;
  const frozenColumns = tabState.frozenColumns;
  const hiddenColumns = tabState.hiddenColumns;

  const stagedChanges = useTabStagedChanges(tabId);
  const [isApplying, setIsApplying] = useState(false);
  const [applyError, setApplyError] = useState<string | null>(null);

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

  /* ---- helpers ---- */

  const colNameToIdx = useMemo(
    () => new Map(columns.map((c, i) => [c.name, i])),
    [columns],
  );

  const getPkValues = useCallback(
    (row: CellValue[]): CellValue[] =>
      pkColumns.map((pkCol) => {
        const idx = colNameToIdx.get(pkCol);
        return idx !== undefined ? row[idx] : { type: "null" as const };
      }),
    [pkColumns, colNameToIdx],
  );

  const handleSort = (column: string) => {
    store.setSorts(tabId, cycleColumnSort(sorts, column));
  };

  const handleRefresh = () => {
    query.refetch();
  };

  // React to header Refresh clicks — refetch rows when the counter increments.
  const prevRefreshCounter = useRef(refreshCounter);
  useEffect(() => {
    if (refreshCounter !== prevRefreshCounter.current) {
      prevRefreshCounter.current = refreshCounter;
      query.refetch();
    }
  }, [refreshCounter, query]);

  /* ---- staged cell edit (no immediate backend call) ---- */

  const handleCellSave = (rowIdx: number, colIdx: number, value: CellValue) => {
    if (!pkColumns.length) return;
    const row = rows[rowIdx];
    const updatedRow = [...row];
    updatedRow[colIdx] = value;
    const pkValues = getPkValues(row);

    useStagedChangesStore.getState().stageCellEdit(tabId, {
      kind: "cell-edit",
      pkValues,
      currentValues: updatedRow,
      columnName: columns[colIdx].name,
      newValue: value,
    });
  };

  /* ---- staged row delete (no immediate backend call) ---- */

  const handleDeleteRow = (rowIdx: number) => {
    if (!pkColumns.length) return;
    const row = rows[rowIdx];
    const pkValues = getPkValues(row);
    useStagedChangesStore.getState().stageDeleteRow(tabId, pkValues);
  };

  /* ---- apply all staged changes ---- */

  const handleApply = useCallback(async () => {
    if (isApplying || stagedChanges.length === 0) return;
    setIsApplying(true);
    setApplyError(null);

    const changes = [...stagedChanges];
    let failed = false;

    for (const change of changes) {
      try {
        if (change.kind === "cell-edit") {
          await updateRow.mutateAsync({
            schema,
            table,
            columns: columns.map((c) => c.name),
            values: change.currentValues,
            pkColumns,
            pkValues: change.pkValues,
          });
        } else if (change.kind === "row-delete") {
          // For delete we need the full row values — find from current data or use pkValues
          await deleteRow.mutateAsync({
            schema,
            table,
            columns: columns.map((c) => c.name),
            values: change.pkValues, // backend only needs PK columns for WHERE clause
            pkColumns,
            pkValues: change.pkValues,
          });
        }
      } catch (err) {
        failed = true;
        const msg = err instanceof Error ? err.message : String(err);
        setApplyError(msg);
        break;
      }
    }

    if (!failed) {
      useStagedChangesStore.getState().clearChanges(tabId);
      query.refetch();
    }

    setIsApplying(false);
  }, [isApplying, stagedChanges, tabId, schema, table, columns, pkColumns, updateRow, deleteRow, query]);

  /* ---- revert all ---- */

  const handleRevertAll = useCallback(() => {
    useStagedChangesStore.getState().revertAll(tabId);
    setApplyError(null);
  }, [tabId]);

  const errorMessage = query.isError
    ? (query.error as { userMessage?: string })?.userMessage ?? t("common.states.error")
    : null;

  return (
    <ObjectSectionLayout
      toolbar={
        <DataToolbar
          columns={columns}
          rowCount={totalCount}
          filters={filters}
          sorts={sorts}
          hiddenColumns={hiddenColumns}
          onAddFilter={(f) => store.addFilter(tabId, f)}
          onRemoveFilter={(i) => store.removeFilter(tabId, i)}
          onSetSorts={(s) => store.setSorts(tabId, s)}
          onToggleHiddenColumn={(c) => store.toggleHiddenColumn(tabId, c)}
          onShowAllColumns={() => store.setHiddenColumns(tabId, [])}
          onRefresh={handleRefresh}
          isRefreshing={query.isRefetching}
        />
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
      {errorMessage ? (
        <div className="flex h-full min-h-0 items-center justify-center p-8">
          <p className="text-sm text-destructive">{errorMessage}</p>
        </div>
      ) : (
        <>
          <ChangeBar
            changes={stagedChanges}
            isApplying={isApplying}
            onApply={handleApply}
            onRevertAll={handleRevertAll}
          />
          {applyError && (
            <div className="mx-3 mt-2 rounded-sm bg-destructive px-3 py-1.5 text-xs text-white">
              {applyError}
            </div>
          )}
          <DataGrid
            columns={columns}
            rows={rows}
            sorts={sorts}
            onSort={handleSort}
            editingCell={editingCell}
            onEditCell={(c) => store.setEditingCell(tabId, c)}
            onCellSave={handleCellSave}
            onDeleteRow={handleDeleteRow}
            isDeleting={isApplying}
            isLoading={query.isFetching && !query.isPlaceholderData}
            pkColumns={pkColumns}
            frozenColumns={frozenColumns}
            hiddenColumns={hiddenColumns}
            onToggleFreezeColumn={(c) => store.toggleFrozenColumn(tabId, c)}
          />
        </>
      )}
    </ObjectSectionLayout>
  );
}
