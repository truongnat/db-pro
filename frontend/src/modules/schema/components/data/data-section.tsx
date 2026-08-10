import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { useTranslation } from "@/commons/locales/useTranslation";

import { useIntrospect } from "@/modules/schema/queries/schema.queries";
import {
  useTableRows,
  useUpdateRow,
  useDeleteRow,
} from "@/modules/data-grid/queries/data-grid.queries";
import { DataGrid } from "@/modules/data-grid/components/data-grid";
import { ChangeBar } from "@/modules/data-grid/components/change-bar";
import { TransactionFeedback } from "@/modules/data-grid/components/transaction-feedback";
import { Pagination } from "@/modules/data-grid/components/pagination";
import { DataToolbar } from "@/modules/data-grid/components/data-toolbar";
import { DeleteConfirmationDialog } from "@/modules/data-grid/components/delete-confirmation-dialog";
import { RowEditDialog } from "@/modules/data-grid/components/row-edit-dialog";
import { useTabGridStateStore } from "@/modules/data-grid/state/tab-grid-state.store";
import {
  useStagedChangesStore,
  useTabStagedChanges,
  pkKey,
} from "@/modules/data-grid/state/staged-changes.store";
import type { StagedChange } from "@/modules/data-grid/state/staged-changes.store";
import { cycleColumnSort } from "@/modules/data-grid/utils/sort";
import { classifyConstraintError } from "@/modules/data-grid/utils/constraint-errors";
import { normalizeMutationError } from "@/modules/data-grid/utils/mutation-error";
import type {
  CellValue,
  FetchRowsRequest,
  GridSort,
} from "@/modules/data-grid/types/data-grid.types";

import { ObjectSectionLayout } from "../object-section-layout";

interface DataSectionProps {
  tabId: string;
  connectionId: string;
  schema: string;
  table: string;
  /** Incremented by the parent when the header Refresh is clicked. */
  refreshCounter?: number;
}

export function DataSection({
  tabId,
  connectionId,
  schema,
  table,
  refreshCounter = 0,
}: DataSectionProps) {
  const { t } = useTranslation();
  const tabState = useTabGridStateStore((s) => s.states[tabId]) ?? {
    filters: [],
    sorts: [] as GridSort[],
    draftFilters: [],
    draftSorts: [] as GridSort[],
    page: 1,
    pageSize: 50,
    editingCell: null,
    frozenColumns: [] as string[],
    hiddenColumns: [] as string[],
    columnWidths: {} as Record<string, number>,
    chartConfig: null,
  };
  const store = useTabGridStateStore.getState();

  const filters = tabState.filters;
  const sorts = tabState.sorts;
  const draftFilters = tabState.draftFilters;
  const draftSorts = tabState.draftSorts;
  const page = tabState.page;
  const pageSize = tabState.pageSize;
  const editingCell = tabState.editingCell;
  const frozenColumns = tabState.frozenColumns;
  const hiddenColumns = tabState.hiddenColumns;
  const columnWidths = tabState.columnWidths;

  const stagedChanges = useTabStagedChanges(tabId);
  const [isApplying, setIsApplying] = useState(false);
  const [applyError, setApplyError] = useState<string | null>(null);
  const [selectedRows, setSelectedRows] = useState<Set<number>>(new Set());
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [editingRowIdx, setEditingRowIdx] = useState<number | null>(null);
  const [transactionResult, setTransactionResult] = useState<{
    kind: "success" | "partial" | "failure";
    succeeded: number;
    failed: number;
    durationMs: number;
  } | null>(null);

  const introspect = useIntrospect(connectionId);

  const pkColumns = useMemo(() => {
    if (!introspect.data) return [];
    return introspect.data.primaryKeys
      .filter((pk) => pk.schema === schema && pk.tableName === table)
      .flatMap((pk) => pk.columns);
  }, [introspect.data, schema, table]);

  const request: FetchRowsRequest = { schema, table, filters, sorts, page, pageSize };

  const query = useTableRows(connectionId, request);
  const updateRow = useUpdateRow(connectionId, request);
  const deleteRow = useDeleteRow(connectionId, request);

  const columns = query.data?.columns ?? [];
  const rows = query.data?.rows ?? [];
  const totalCount = query.data?.totalCount ?? 0;

  /* ---- helpers ---- */

  const colNameToIdx = useMemo(() => new Map(columns.map((c, i) => [c.name, i])), [columns]);

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

  // React to header Refresh clicks — refetch rows when the counter increments.
  const prevRefreshCounter = useRef(refreshCounter);
  useEffect(() => {
    if (refreshCounter !== prevRefreshCounter.current) {
      prevRefreshCounter.current = refreshCounter;
      query.refetch();
    }
  }, [refreshCounter, query]);

  /* ---- staged cell edit (patch model — no immediate backend call) ---- */

  const handleCellSave = (rowIdx: number, colIdx: number, value: CellValue) => {
    if (!pkColumns.length) return;
    const colName = columns[colIdx].name;
    const row = rows[rowIdx];
    const pkValues = getPkValues(row);

    // Compose patch: start from base row, overlay existing staged changes, then new edit
    const existingChange = stagedChanges.find(
      (c) => c.kind === "cell-edit" && pkKey(c.pkValues) === pkKey(pkValues),
    );
    const changes: Record<string, CellValue> =
      existingChange?.kind === "cell-edit" ? { ...existingChange.changes } : {};
    changes[colName] = value;

    useStagedChangesStore.getState().stageCellEdit(tabId, { pkValues, changes });
  };

  /* ---- staged row delete (no immediate backend call) ---- */

  const handleDeleteRow = (rowIdx: number) => {
    if (!pkColumns.length) return;
    const row = rows[rowIdx];
    const pkValues = getPkValues(row);
    useStagedChangesStore.getState().stageDeleteRow(tabId, pkValues);
  };

  /* ---- apply staged changes (patch model + revision-safe) ---- */

  const applyChanges = useCallback(
    async (changesToApply: StagedChange[]) => {
      if (isApplying || changesToApply.length === 0) return;
      setIsApplying(true);
      setApplyError(null);

      // Mark these revisions as in-flight so new edits create new revisions
      const snapshotIds = changesToApply.map((c) => c.id);
      useStagedChangesStore.getState().markInFlight(tabId, snapshotIds);

      const successIds: string[] = [];
      const failures: Array<{ id: string; error: string }> = [];
      const startTime = Date.now();

      for (const change of changesToApply) {
        try {
          if (change.kind === "cell-edit") {
            const changedCols = Object.keys(change.changes);
            await updateRow.mutateAsync({
              schema,
              table,
              columns: changedCols,
              values: changedCols.map((col) => change.changes[col]),
              pkColumns,
              pkValues: change.pkValues,
            });
          } else if (change.kind === "row-delete") {
            await deleteRow.mutateAsync({
              schema,
              table,
              columns: columns.map((c) => c.name),
              values: change.pkValues,
              pkColumns,
              pkValues: change.pkValues,
            });
          }
          successIds.push(change.id);
        } catch (err) {
          const normalized = normalizeMutationError(err);
          const classified = classifyConstraintError(normalized.technicalMessage, normalized.details);
          failures.push({
            id: change.id,
            error: classified.userMessage,
          });
        }
      }

      const durationMs = Date.now() - startTime;

      if (failures.length === 0) {
        useStagedChangesStore.getState().removeByIds(tabId, successIds);
        query.refetch();
        setTransactionResult({ kind: "success", succeeded: successIds.length, failed: 0, durationMs });
      } else if (successIds.length > 0) {
        useStagedChangesStore.getState().removeByIds(tabId, successIds);
        useStagedChangesStore.getState().markFailedByIds(tabId, failures);
        setApplyError(failures[0].error);
        query.refetch();
        setTransactionResult({ kind: "partial", succeeded: successIds.length, failed: failures.length, durationMs });
      } else {
        setApplyError(failures[0].error);
        useStagedChangesStore.getState().markFailedByIds(tabId, failures);
        setTransactionResult({ kind: "failure", succeeded: 0, failed: failures.length, durationMs });
      }

      setIsApplying(false);
      setSelectedRows(new Set());
    },
    [isApplying, tabId, schema, table, columns, pkColumns, updateRow, deleteRow, query],
  );

  const handleApply = useCallback(() => {
    if (isApplying || stagedChanges.length === 0) return;

    const deletes = stagedChanges.filter((c) => c.kind === "row-delete").length;
    if (deletes > 0) {
      setDeleteDialogOpen(true);
      return;
    }

    applyChanges(stagedChanges);
  }, [isApplying, stagedChanges, applyChanges]);

  const handleConfirmApply = useCallback(() => {
    setDeleteDialogOpen(false);
    applyChanges(stagedChanges);
  }, [stagedChanges, applyChanges]);

  /* ---- retry failed ---- */

  const handleRetryFailed = useCallback(async () => {
    const failedChanges = stagedChanges.filter((c) => "error" in c && c.error);
    if (failedChanges.length === 0) return;
    // Clear error flags before retry
    useStagedChangesStore.getState().clearFailed(tabId);
    await applyChanges(failedChanges);
  }, [stagedChanges, tabId, applyChanges]);

  /* ---- revert all ---- */

  const handleRevertAll = useCallback(() => {
    useStagedChangesStore.getState().clearTab(tabId);
    setApplyError(null);
    setTransactionResult(null);
    setSelectedRows(new Set());
  }, [tabId]);

  /* ---- Cmd/Ctrl+Enter: apply staged changes ---- */

  const handleGridKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "Enter" && stagedChanges.length > 0 && !isApplying) {
        e.preventDefault();
        handleApply();
      }
    },
    [stagedChanges.length, isApplying, handleApply],
  );

  /* ---- batch delete selected rows ---- */

  const handleBatchDelete = useCallback(
    (selected: Set<number>) => {
      if (!pkColumns.length || selected.size === 0) return;
      for (const rowIdx of selected) {
        const row = rows[rowIdx];
        if (row) {
          const pkValues = getPkValues(row);
          useStagedChangesStore.getState().stageDeleteRow(tabId, pkValues);
        }
      }
      setSelectedRows(new Set());
    },
    [pkColumns, rows, getPkValues, tabId],
  );

  /* ---- row edit dialog save ---- */

  const handleRowSave = useCallback(
    (changes: Record<string, CellValue>) => {
      if (editingRowIdx == null || !pkColumns.length) return;
      const row = rows[editingRowIdx];
      if (!row) return;
      const pkValues = getPkValues(row);
      useStagedChangesStore.getState().stageCellEdit(tabId, { pkValues, changes });
      setEditingRowIdx(null);
    },
    [editingRowIdx, pkColumns, rows, getPkValues, tabId],
  );

  const errorMessage = query.isError
    ? ((query.error as { userMessage?: string })?.userMessage ?? t("common.states.error"))
    : null;

  return (
    <ObjectSectionLayout
      toolbar={
        <DataToolbar
          columns={columns}
          rowCount={totalCount}
          filters={filters}
          sorts={sorts}
          draftFilters={draftFilters}
          draftSorts={draftSorts}
          hiddenColumns={hiddenColumns}
          onAddDraftFilter={(f) => store.addDraftFilter(tabId, f)}
          onRemoveDraftFilter={(i) => store.removeDraftFilter(tabId, i)}
          onApplyFilters={() => store.applyFilters(tabId)}
          onClearFilters={() => store.clearFilters(tabId)}
          onAddDraftSort={(s) => store.addDraftSort(tabId, s)}
          onRemoveDraftSort={(i) => store.removeDraftSort(tabId, i)}
          onApplySorts={() => store.applySorts(tabId)}
          onClearSorts={() => store.clearSorts(tabId)}
          onToggleHiddenColumn={(c) => store.toggleHiddenColumn(tabId, c)}
          onShowAllColumns={() => store.setHiddenColumns(tabId, [])}
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
            onRetryFailed={handleRetryFailed}
            onBatchDelete={handleBatchDelete}
            selectedRows={selectedRows}
          />
          <DeleteConfirmationDialog
            open={deleteDialogOpen}
            onOpenChange={setDeleteDialogOpen}
            onConfirm={handleConfirmApply}
            deleteCount={stagedChanges.filter((c) => c.kind === "row-delete").length}
            totalChanges={stagedChanges.length}
          />
          <TransactionFeedback result={transactionResult} onDismiss={() => setTransactionResult(null)} />
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
            onEditRow={(idx) => setEditingRowIdx(idx)}
            isDeleting={isApplying}
            isLoading={query.isFetching && !query.isPlaceholderData}
            pkColumns={pkColumns}
            frozenColumns={frozenColumns}
            hiddenColumns={hiddenColumns}
            onToggleFreezeColumn={(c) => store.toggleFrozenColumn(tabId, c)}
            columnWidths={columnWidths}
            onColumnWidthsChange={(w) => store.setColumnWidths(tabId, w)}
            onKeyDown={handleGridKeyDown}
            selectedRows={selectedRows}
            onSelectionChange={setSelectedRows}
          />
          {editingRowIdx != null && rows[editingRowIdx] && (
            <RowEditDialog
              open={editingRowIdx != null}
              onOpenChange={(open) => { if (!open) setEditingRowIdx(null); }}
              columns={columns}
              row={rows[editingRowIdx]}
              onSave={handleRowSave}
            />
          )}
        </>
      )}
    </ObjectSectionLayout>
  );
}
