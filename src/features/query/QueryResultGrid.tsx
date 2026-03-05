import { useEffect, useState } from "react";
import {
  ArrowDownAZ,
  ArrowUpZA,
  ChevronLeft,
  ChevronRight,
  Copy,
  Download,
  Rows3,
  Search,
  X,
} from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import type { QueryResult } from "@/types";
import type { GridSelectionSummary } from "./grid-selection";
import { VirtualizedResultTable } from "./VirtualizedResultTable";

const PAGE_SIZE_OPTIONS = [100, 250, 500, 1000, 2000] as const;

export type QueryGridModifiers = {
  quickFilter: string;
  sortColumn: string;
  sortDirection: "asc" | "desc";
};

type QueryResultGridProps = {
  queryResult: QueryResult | null;
  loadingPage: boolean;
  queryPageSize: number;
  gridModifiers: QueryGridModifiers;
  onQueryPageSizeChange: (nextPageSize: number) => void;
  onGridModifiersChange: (next: QueryGridModifiers) => void;
  onCopyResultPage: (columns: string[], rows: string[][]) => void;
  onExportResultCsv: (columns: string[], rows: string[][]) => void;
  onGridCopyFeedback: (message: string) => void;
  onPreviousPage: () => void;
  onNextPage: () => void;
};

export function QueryResultGrid({
  queryResult,
  loadingPage,
  queryPageSize,
  gridModifiers,
  onQueryPageSizeChange,
  onGridModifiersChange,
  onCopyResultPage,
  onExportResultCsv,
  onGridCopyFeedback,
  onPreviousPage,
  onNextPage,
}: QueryResultGridProps) {
  const [selectionSummary, setSelectionSummary] = useState<GridSelectionSummary | null>(null);

  useEffect(() => {
    if (!queryResult || queryResult.rows.length === 0 || queryResult.columns.length === 0) {
      setSelectionSummary(null);
    }
  }, [queryResult]);

  const canPaginate = !!queryResult?.isRowQuery && queryResult.pageSize > 0;
  const hasPreviousPage = canPaginate && queryResult.pageOffset > 0;
  const hasNextPage = canPaginate && queryResult.hasMore;

  const hasRows = !!queryResult && queryResult.columns.length > 0 && queryResult.rows.length > 0;

  const hasFilter = gridModifiers.quickFilter.trim().length > 0;
  const hasSort = gridModifiers.sortColumn.length > 0;

  const rangeLabel = (() => {
    if (!canPaginate || !queryResult) {
      return null;
    }

    if (queryResult.rows.length === 0) {
      return "No rows on this page";
    }

    const from = queryResult.pageOffset + 1;
    const to = queryResult.pageOffset + queryResult.rows.length;
    return `Rows ${from}-${to}`;
  })();

  return (
    <div className="flex h-full min-h-0 flex-col gap-2">
      <div className="shrink-0 flex flex-wrap items-center justify-between gap-2 text-xs text-muted-foreground">
        <div className="flex items-center gap-2">
          <Rows3 className="h-4 w-4" />
          <span>
            {queryResult
              ? `${queryResult.rows.length} row(s), ${queryResult.columns.length} column(s)`
              : "Run a query to inspect data"}
          </span>
          {rangeLabel && <Badge variant="outline">{rangeLabel}</Badge>}
          {canPaginate && queryResult.hasMore && (
            <Badge variant="secondary">More rows available</Badge>
          )}
          {hasFilter && <Badge variant="secondary">Filtered</Badge>}
          {hasSort && <Badge variant="secondary">Sorted</Badge>}
          {selectionSummary && selectionSummary.cellCount > 0 && (
            <Badge variant="secondary">
              Selection {selectionSummary.rowCount}x{selectionSummary.columnCount}
            </Badge>
          )}
        </div>

        <div className="flex items-center gap-2">
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={() => {
              if (!queryResult) {
                return;
              }
              onCopyResultPage(queryResult.columns, queryResult.rows);
            }}
            disabled={loadingPage || !hasRows}
          >
            <Copy className="mr-1 h-3.5 w-3.5" />
            Copy Page
          </Button>
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={() => {
              if (!queryResult) {
                return;
              }
              onExportResultCsv(queryResult.columns, queryResult.rows);
            }}
            disabled={loadingPage || !hasRows}
          >
            <Download className="mr-1 h-3.5 w-3.5" />
            Export CSV
          </Button>

          <label
            htmlFor="query-page-size"
            className="hidden text-[11px] text-muted-foreground sm:inline"
          >
            Rows / page
          </label>
          <select
            id="query-page-size"
            value={queryPageSize}
            onChange={(event) => {
              const nextValue = Number.parseInt(event.target.value, 10);
              if (Number.isFinite(nextValue) && nextValue > 0) {
                onQueryPageSizeChange(nextValue);
              }
            }}
            disabled={loadingPage}
            className="h-8 rounded-md border bg-background px-2 text-xs text-foreground shadow-sm outline-none ring-offset-background focus-visible:ring-2 focus-visible:ring-ring"
          >
            {PAGE_SIZE_OPTIONS.map((option) => (
              <option key={option} value={option}>
                {option}
              </option>
            ))}
          </select>

          {canPaginate && (
            <>
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={onPreviousPage}
                disabled={loadingPage || !hasPreviousPage}
              >
                <ChevronLeft className="mr-1 h-3.5 w-3.5" />
                Prev
              </Button>
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={onNextPage}
                disabled={loadingPage || !hasNextPage}
              >
                Next
                <ChevronRight className="ml-1 h-3.5 w-3.5" />
              </Button>
            </>
          )}
          {queryResult && <Badge variant="secondary">{queryResult.executionMs} ms</Badge>}
        </div>
      </div>

      {queryResult && queryResult.columns.length > 0 && (
        <div className="shrink-0 flex flex-wrap items-center gap-2 rounded-md border border-border/70 bg-muted/25 px-2 py-2">
          <div className="relative min-w-[220px] flex-1">
            <Search className="pointer-events-none absolute left-2 top-2 h-3.5 w-3.5 text-muted-foreground" />
            <Input
              value={gridModifiers.quickFilter}
              onChange={(event) =>
                onGridModifiersChange({
                  ...gridModifiers,
                  quickFilter: event.target.value,
                })
              }
              placeholder="Quick filter rows (server)..."
              className="h-8 pl-7 text-xs"
            />
          </div>

          <select
            value={gridModifiers.sortColumn}
            onChange={(event) =>
              onGridModifiersChange({
                ...gridModifiers,
                sortColumn: event.target.value,
              })
            }
            className="h-8 rounded-md border bg-background px-2 text-xs text-foreground shadow-sm outline-none ring-offset-background focus-visible:ring-2 focus-visible:ring-ring"
          >
            <option value="">Sort: none</option>
            {queryResult.columns.map((column) => (
              <option key={column} value={column}>
                {column}
              </option>
            ))}
          </select>

          <Button
            type="button"
            variant="outline"
            size="sm"
            className="h-8"
            disabled={!gridModifiers.sortColumn}
            onClick={() =>
              onGridModifiersChange({
                ...gridModifiers,
                sortDirection:
                  gridModifiers.sortDirection === "asc" ? "desc" : "asc",
              })
            }
          >
            {gridModifiers.sortDirection === "asc" ? (
              <ArrowDownAZ className="mr-1 h-3.5 w-3.5" />
            ) : (
              <ArrowUpZA className="mr-1 h-3.5 w-3.5" />
            )}
            {gridModifiers.sortDirection.toUpperCase()}
          </Button>

          <Button
            type="button"
            variant="ghost"
            size="sm"
            className="h-8"
            disabled={!hasFilter && !hasSort}
            onClick={() => {
              onGridModifiersChange({
                quickFilter: "",
                sortColumn: "",
                sortDirection: "asc",
              });
            }}
          >
            <X className="mr-1 h-3.5 w-3.5" />
            Clear
          </Button>
        </div>
      )}

      {!queryResult && (
        <div className="shrink-0 rounded-md border border-dashed bg-muted/50 p-4 text-sm text-muted-foreground">
          No query result yet.
        </div>
      )}

      {queryResult && queryResult.columns.length === 0 && (
        <div className="shrink-0 rounded-md border border-dashed bg-muted/50 p-4 text-sm text-muted-foreground">
          {queryResult.message}
        </div>
      )}

      {queryResult && queryResult.columns.length > 0 && queryResult.rows.length === 0 && (
        <div className="shrink-0 rounded-md border border-dashed bg-muted/50 p-4 text-sm text-muted-foreground">
          {hasFilter ? "No rows match current filter." : "No rows on this page."}
        </div>
      )}

      {queryResult && queryResult.columns.length > 0 && queryResult.rows.length > 0 && (
        <div className="min-h-0 flex-1 overflow-hidden rounded-md border bg-background">
          <VirtualizedResultTable
            columns={queryResult.columns}
            rows={queryResult.rows}
            onCopyFeedback={onGridCopyFeedback}
            onSelectionSummaryChange={setSelectionSummary}
          />
        </div>
      )}

      {queryResult && queryResult.columns.length > 0 && queryResult.rows.length > 0 && (
        <div className="shrink-0 text-[11px] text-muted-foreground">
          Shortcuts: <kbd className="rounded border px-1 py-0.5">Ctrl/Cmd+C</kbd> copy TSV,{" "}
          <kbd className="rounded border px-1 py-0.5">Ctrl/Cmd+Shift+C</kbd> copy CSV.
        </div>
      )}
    </div>
  );
}
