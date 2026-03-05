import { useCallback, useEffect, useRef, useState } from "react";

import { loadCachedQueryPageSize, saveCachedQueryPageSize } from "@/features/connections/cache";
import type { QueryResult } from "@/types";

import type { RunQueryPageFn } from "./controller-types";
import type { QueryGridModifiers } from "./QueryResultGrid";

const DEFAULT_QUERY_PAGE_SIZE = 500;
const FILTER_DEBOUNCE_MS = 300;
const DEFAULT_QUERY_GRID_MODIFIERS: QueryGridModifiers = {
  quickFilter: "",
  sortColumn: "",
  sortDirection: "asc",
};

type UseResultGridControllerOptions = {
  selectedConnectionId: string | null;
  busyAction: string | null;
  activeQuerySql: string | null;
  queryResult: QueryResult | null;
  onStatus: (message: string) => void;
  onRunQueryPage: RunQueryPageFn;
};

export function useResultGridController({
  selectedConnectionId,
  busyAction,
  activeQuerySql,
  queryResult,
  onStatus,
  onRunQueryPage,
}: UseResultGridControllerOptions) {
  const [queryPageSize, setQueryPageSize] = useState(() =>
    loadCachedQueryPageSize(DEFAULT_QUERY_PAGE_SIZE),
  );
  const [queryGridModifiers, setQueryGridModifiers] = useState<QueryGridModifiers>(
    DEFAULT_QUERY_GRID_MODIFIERS,
  );
  const filterDebounceTimerRef = useRef<number | null>(null);
  const appliedGridModifiersRef = useRef<QueryGridModifiers>(
    DEFAULT_QUERY_GRID_MODIFIERS,
  );

  const clearFilterDebounceTimer = useCallback(() => {
    if (filterDebounceTimerRef.current !== null) {
      window.clearTimeout(filterDebounceTimerRef.current);
      filterDebounceTimerRef.current = null;
    }
  }, []);

  const resetQueryGridModifiers = useCallback(() => {
    clearFilterDebounceTimer();
    setQueryGridModifiers(DEFAULT_QUERY_GRID_MODIFIERS);
    appliedGridModifiersRef.current = DEFAULT_QUERY_GRID_MODIFIERS;
  }, [clearFilterDebounceTimer]);

  useEffect(() => {
    return () => {
      if (filterDebounceTimerRef.current !== null) {
        window.clearTimeout(filterDebounceTimerRef.current);
      }
    };
  }, []);

  useEffect(() => {
    saveCachedQueryPageSize(queryPageSize);
  }, [queryPageSize]);

  useEffect(() => {
    resetQueryGridModifiers();
  }, [resetQueryGridModifiers, selectedConnectionId]);

  const handleGridModifiersChange = useCallback((next: QueryGridModifiers) => {
    setQueryGridModifiers(next);
  }, []);

  const handleNextPage = useCallback(async () => {
    if (!selectedConnectionId) {
      onStatus("Select a connection first.");
      return;
    }

    if (!queryResult?.isRowQuery || !activeQuerySql) {
      onStatus("Run a row query first.");
      return;
    }

    if (!queryResult.hasMore) {
      onStatus("No more rows.");
      return;
    }

    const nextOffset = queryResult.pageOffset + queryResult.pageSize;
    await onRunQueryPage(activeQuerySql, nextOffset, "next page");
  }, [activeQuerySql, onRunQueryPage, onStatus, queryResult, selectedConnectionId]);

  const handlePreviousPage = useCallback(async () => {
    if (!selectedConnectionId) {
      onStatus("Select a connection first.");
      return;
    }

    if (!queryResult?.isRowQuery || !activeQuerySql) {
      onStatus("Run a row query first.");
      return;
    }

    if (queryResult.pageOffset === 0) {
      onStatus("Already at the first page.");
      return;
    }

    const previousOffset = Math.max(queryResult.pageOffset - queryResult.pageSize, 0);
    await onRunQueryPage(activeQuerySql, previousOffset, "previous page");
  }, [activeQuerySql, onRunQueryPage, onStatus, queryResult, selectedConnectionId]);

  const handleQueryPageSizeChange = useCallback(
    async (nextPageSize: number) => {
      if (!Number.isFinite(nextPageSize) || nextPageSize <= 0) {
        return;
      }
      if (nextPageSize === queryPageSize) {
        return;
      }

      setQueryPageSize(nextPageSize);
      onStatus(`Query page size set to ${nextPageSize}.`);

      if (!selectedConnectionId) {
        return;
      }
      if (!queryResult?.isRowQuery || !activeQuerySql) {
        return;
      }

      await onRunQueryPage(activeQuerySql, 0, "page size change", {
        pageSizeOverride: nextPageSize,
      });
    },
    [
      activeQuerySql,
      onRunQueryPage,
      onStatus,
      queryPageSize,
      queryResult,
      selectedConnectionId,
    ],
  );

  useEffect(() => {
    if (!selectedConnectionId) {
      return;
    }
    if (!queryResult?.isRowQuery || !activeQuerySql) {
      return;
    }
    if (busyAction === "run-query") {
      return;
    }
    if (areGridModifiersEqual(queryGridModifiers, appliedGridModifiersRef.current)) {
      return;
    }

    clearFilterDebounceTimer();
    filterDebounceTimerRef.current = window.setTimeout(() => {
      void onRunQueryPage(activeQuerySql, 0, "filter/sort", {
        gridModifiersOverride: queryGridModifiers,
        skipHistory: true,
      });
    }, FILTER_DEBOUNCE_MS);

    return () => {
      clearFilterDebounceTimer();
    };
  }, [
    activeQuerySql,
    busyAction,
    clearFilterDebounceTimer,
    onRunQueryPage,
    queryGridModifiers,
    queryResult?.isRowQuery,
    selectedConnectionId,
  ]);

  const markGridModifiersApplied = useCallback((applied: QueryGridModifiers) => {
    appliedGridModifiersRef.current = applied;
  }, []);

  const resetForDataReset = useCallback(() => {
    clearFilterDebounceTimer();
    setQueryPageSize(DEFAULT_QUERY_PAGE_SIZE);
    setQueryGridModifiers(DEFAULT_QUERY_GRID_MODIFIERS);
    appliedGridModifiersRef.current = DEFAULT_QUERY_GRID_MODIFIERS;
  }, [clearFilterDebounceTimer]);

  return {
    queryPageSize,
    queryGridModifiers,
    handleGridModifiersChange,
    resetQueryGridModifiers,
    handleNextPage,
    handlePreviousPage,
    handleQueryPageSizeChange,
    markGridModifiersApplied,
    resetForDataReset,
  };
}

function areGridModifiersEqual(
  left: QueryGridModifiers,
  right: QueryGridModifiers,
): boolean {
  return (
    left.quickFilter === right.quickFilter &&
    left.sortColumn === right.sortColumn &&
    left.sortDirection === right.sortDirection
  );
}
