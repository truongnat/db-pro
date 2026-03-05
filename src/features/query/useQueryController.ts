import {
  type Dispatch,
  type SetStateAction,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";

import { cancelQuery, executeQuery } from "@/lib/tauri";
import { loadCachedQueryPageSize, saveCachedQueryPageSize } from "@/features/connections/cache";
import type { ConnectionListItem, QueryResult } from "@/types";

import { copyTextToClipboard, toTabDelimited } from "./clipboard";
import { buildResultCsvFilename, downloadCsv, toCsv } from "./csv";
import { isTimeoutError } from "./errors";
import {
  appendQueryHistory,
  clearQueryHistory as clearQueryHistoryStorage,
  loadQueryHistory,
  saveQueryHistory,
  type QueryHistoryEntry,
} from "./history";
import type { QueryGridModifiers } from "./QueryResultGrid";
import { detectSqlTarget } from "../sql-editor/statement";
import { formatSqlText } from "../sql-editor/format";
import type { SqlEditorSelection } from "../sql-editor/types";

const DEFAULT_QUERY_PAGE_SIZE = 500;
const QUERY_FRONTEND_TIMEOUT_MS = 45_000;
const FILTER_DEBOUNCE_MS = 300;
const DEFAULT_QUERY_GRID_MODIFIERS: QueryGridModifiers = {
  quickFilter: "",
  sortColumn: "",
  sortDirection: "asc",
};

type RunQueryPageOptions = {
  pageSizeOverride?: number;
  gridModifiersOverride?: QueryGridModifiers;
  skipHistory?: boolean;
};

type UseQueryControllerOptions = {
  initialSqlText: string;
  selectedConnectionId: string | null;
  selectedConnection: ConnectionListItem | null;
  busyAction: string | null;
  onSetBusyAction: Dispatch<SetStateAction<string | null>>;
  onStatus: (message: string) => void;
  onClearError: () => void;
  onError: (error: unknown) => void;
  onRefreshNavigator: (connectionId?: string) => void | Promise<void>;
};

export function useQueryController({
  initialSqlText,
  selectedConnectionId,
  selectedConnection,
  busyAction,
  onSetBusyAction,
  onStatus,
  onClearError,
  onError,
  onRefreshNavigator,
}: UseQueryControllerOptions) {
  const [sqlText, setSqlText] = useState(initialSqlText);
  const [queryResult, setQueryResult] = useState<QueryResult | null>(null);
  const [activeQuerySql, setActiveQuerySql] = useState<string | null>(null);
  const [queryHistory, setQueryHistory] = useState<QueryHistoryEntry[]>(() =>
    loadQueryHistory(),
  );
  const [queryPageSize, setQueryPageSize] = useState(() =>
    loadCachedQueryPageSize(DEFAULT_QUERY_PAGE_SIZE),
  );
  const [queryGridModifiers, setQueryGridModifiers] = useState<QueryGridModifiers>(
    DEFAULT_QUERY_GRID_MODIFIERS,
  );
  const [cancelRequested, setCancelRequested] = useState(false);

  const sqlSelectionRef = useRef<SqlEditorSelection | null>(null);
  const filterDebounceTimerRef = useRef<number | null>(null);
  const appliedGridModifiersRef = useRef<QueryGridModifiers>(
    DEFAULT_QUERY_GRID_MODIFIERS,
  );
  const queryRequestIdRef = useRef(0);

  const queryHistoryForSelectedConnection = useMemo(() => {
    if (!selectedConnectionId) {
      return [];
    }

    return queryHistory
      .filter((entry) => entry.connectionId === selectedConnectionId)
      .slice(0, 50);
  }, [queryHistory, selectedConnectionId]);

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

  const invalidateQueryRequests = useCallback(() => {
    queryRequestIdRef.current += 1;
  }, []);

  useEffect(() => {
    return () => {
      if (filterDebounceTimerRef.current !== null) {
        window.clearTimeout(filterDebounceTimerRef.current);
      }
    };
  }, []);

  useEffect(() => {
    invalidateQueryRequests();
    clearFilterDebounceTimer();
    setQueryGridModifiers(DEFAULT_QUERY_GRID_MODIFIERS);
    appliedGridModifiersRef.current = DEFAULT_QUERY_GRID_MODIFIERS;
    setQueryResult(null);
    setActiveQuerySql(null);
    onSetBusyAction((previous) => (previous === "run-query" ? null : previous));
    setCancelRequested(false);
  }, [
    clearFilterDebounceTimer,
    invalidateQueryRequests,
    onSetBusyAction,
    selectedConnectionId,
  ]);

  useEffect(() => {
    saveCachedQueryPageSize(queryPageSize);
  }, [queryPageSize]);

  useEffect(() => {
    saveQueryHistory(queryHistory);
  }, [queryHistory]);

  const runQueryPage = useCallback(
    async (
      sql: string,
      offset: number,
      runSource: string,
      options?: RunQueryPageOptions,
    ) => {
      if (!selectedConnectionId) {
        onStatus("Select a connection first.");
        return;
      }

      const effectivePageSize = options?.pageSizeOverride ?? queryPageSize;
      const effectiveGridModifiers =
        options?.gridModifiersOverride ?? queryGridModifiers;
      const normalizedFilter = effectiveGridModifiers.quickFilter.trim();
      const normalizedSortColumn = effectiveGridModifiers.sortColumn.trim();
      const filterColumns =
        normalizedFilter && queryResult?.columns.length
          ? queryResult.columns
          : undefined;
      const requestId = queryRequestIdRef.current + 1;
      queryRequestIdRef.current = requestId;

      try {
        setCancelRequested(false);
        onSetBusyAction("run-query");
        const result = await withPromiseTimeout(
          executeQuery({
            connectionId: selectedConnectionId,
            sql,
            pageSize: effectivePageSize,
            offset,
            timeoutMs: 30000,
            quickFilter: normalizedFilter || undefined,
            filterColumns,
            sortColumn: normalizedSortColumn || undefined,
            sortDirection: normalizedSortColumn
              ? effectiveGridModifiers.sortDirection
              : undefined,
          }),
          QUERY_FRONTEND_TIMEOUT_MS,
          `Query request timed out after ${QUERY_FRONTEND_TIMEOUT_MS} ms`,
        );
        if (queryRequestIdRef.current !== requestId) {
          return;
        }

        const resolvedColumns =
          result.isRowQuery &&
          result.columns.length === 0 &&
          activeQuerySql === sql &&
          queryResult?.columns.length
            ? queryResult.columns
            : result.columns;

        setQueryResult({
          ...result,
          columns: resolvedColumns,
        });
        setActiveQuerySql(result.isRowQuery ? sql : null);
        appliedGridModifiersRef.current = result.isRowQuery
          ? effectiveGridModifiers
          : DEFAULT_QUERY_GRID_MODIFIERS;

        onStatus(`${result.message} in ${result.executionMs} ms (${runSource}).`);
        onClearError();
        if (!options?.skipHistory) {
          setQueryHistory((previous) =>
            appendQueryHistory(previous, {
              id: createQueryHistoryId(),
              connectionId: selectedConnectionId,
              connectionName: selectedConnection?.name ?? "Unknown connection",
              sql,
              runSource,
              executionMs: result.executionMs,
              recordedAt: new Date().toISOString(),
            }),
          );
        }

        if (result.schemaChanged) {
          void onRefreshNavigator(selectedConnectionId);
        }
      } catch (error) {
        if (queryRequestIdRef.current !== requestId) {
          return;
        }
        if (isTimeoutError(error)) {
          void cancelQuery(selectedConnectionId).catch(() => undefined);
        }
        onError(error);
      } finally {
        if (queryRequestIdRef.current === requestId) {
          onSetBusyAction(null);
          setCancelRequested(false);
        }
      }
    },
    [
      activeQuerySql,
      onClearError,
      onError,
      onRefreshNavigator,
      onSetBusyAction,
      onStatus,
      queryGridModifiers,
      queryPageSize,
      queryResult?.columns,
      selectedConnection,
      selectedConnectionId,
    ],
  );

  const runCurrentSql = useCallback(async () => {
    if (!selectedConnectionId) {
      onStatus("Select a connection first.");
      return;
    }

    const sqlTarget = detectSqlTarget(sqlText, sqlSelectionRef.current);
    if (!sqlTarget.sql.trim()) {
      onStatus("SQL is empty.");
      return;
    }

    const runSource =
      sqlTarget.source === "selection"
        ? "selection"
        : sqlTarget.source === "current"
          ? "current statement"
          : "full editor";
    resetQueryGridModifiers();
    await runQueryPage(sqlTarget.sql, 0, runSource, {
      gridModifiersOverride: DEFAULT_QUERY_GRID_MODIFIERS,
    });
  }, [onStatus, resetQueryGridModifiers, runQueryPage, selectedConnectionId, sqlText]);

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
    await runQueryPage(activeQuerySql, nextOffset, "next page");
  }, [activeQuerySql, onStatus, queryResult, runQueryPage, selectedConnectionId]);

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
    await runQueryPage(activeQuerySql, previousOffset, "previous page");
  }, [activeQuerySql, onStatus, queryResult, runQueryPage, selectedConnectionId]);

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

      await runQueryPage(activeQuerySql, 0, "page size change", {
        pageSizeOverride: nextPageSize,
      });
    },
    [activeQuerySql, onStatus, queryPageSize, queryResult, runQueryPage, selectedConnectionId],
  );

  const cancelRunningQuery = useCallback(async () => {
    if (!selectedConnectionId) {
      onStatus("Select a connection first.");
      return;
    }

    if (busyAction !== "run-query") {
      onStatus("No running query to cancel.");
      return;
    }

    try {
      setCancelRequested(true);
      const result = await cancelQuery(selectedConnectionId);
      onStatus(result);
      onClearError();
    } catch (error) {
      onError(error);
      setCancelRequested(false);
    }
  }, [
    busyAction,
    onClearError,
    onError,
    onStatus,
    selectedConnectionId,
  ]);

  const applyQueryHistory = useCallback(
    (historyId: string) => {
      const item = queryHistoryForSelectedConnection.find(
        (entry) => entry.id === historyId,
      );
      if (!item) {
        onStatus("History entry not found.");
        return;
      }

      setSqlText(item.sql);
      onStatus(`Loaded query from history (${item.runSource}, ${item.executionMs} ms).`);
      onClearError();
    },
    [onClearError, onStatus, queryHistoryForSelectedConnection],
  );

  const runQueryHistory = useCallback(
    async (historyId: string) => {
      const item = queryHistoryForSelectedConnection.find(
        (entry) => entry.id === historyId,
      );
      if (!item) {
        onStatus("History entry not found.");
        return;
      }

      setSqlText(item.sql);
      resetQueryGridModifiers();
      await runQueryPage(item.sql, 0, "history", {
        gridModifiersOverride: DEFAULT_QUERY_GRID_MODIFIERS,
      });
      onClearError();
    },
    [
      onClearError,
      onStatus,
      queryHistoryForSelectedConnection,
      resetQueryGridModifiers,
      runQueryPage,
    ],
  );

  const clearSelectedConnectionHistory = useCallback(() => {
    if (!selectedConnectionId) {
      onStatus("Select a connection first.");
      return;
    }

    setQueryHistory((previous) =>
      previous.filter((entry) => entry.connectionId !== selectedConnectionId),
    );
    onStatus("Cleared query history for current connection.");
  }, [onStatus, selectedConnectionId]);

  const applySqlTemplate = useCallback(
    (sql: string, label: string) => {
      const nextText = sqlText.trim() ? `${sqlText.trimEnd()}\n\n${sql}` : sql;
      setSqlText(nextText);
      onStatus(`Inserted SQL template: ${label}.`);
      onClearError();
    },
    [onClearError, onStatus, sqlText],
  );

  const formatSql = useCallback(async () => {
    const nextSql = await formatSqlText(sqlText, selectedConnection?.engine);
    if (nextSql === sqlText) {
      onStatus("SQL is already formatted.");
      onClearError();
      return;
    }

    setSqlText(nextSql);
    onStatus("SQL formatted.");
    onClearError();
  }, [onClearError, onStatus, selectedConnection?.engine, sqlText]);

  const copyResultPage = useCallback(
    async (columns: string[], rows: string[][]) => {
      if (columns.length === 0) {
        onStatus("No result rows to copy.");
        return;
      }

      try {
        const payload = toTabDelimited(columns, rows, true);
        await copyTextToClipboard(payload);
        onStatus(`Copied ${rows.length} row(s) from current page.`);
        onClearError();
      } catch (error) {
        onError(error);
      }
    },
    [onClearError, onError, onStatus],
  );

  const exportResultCsv = useCallback(
    (columns: string[], rows: string[][]) => {
      if (columns.length === 0) {
        onStatus("No result rows to export.");
        return;
      }

      try {
        const csv = toCsv(columns, rows);
        const filename = buildResultCsvFilename(selectedConnection?.name);
        downloadCsv(filename, csv);
        onStatus(`Exported ${rows.length} row(s) to ${filename}.`);
        onClearError();
      } catch (error) {
        onError(error);
      }
    },
    [onClearError, onError, onStatus, selectedConnection?.name],
  );

  const handleGridCopyFeedback = useCallback(
    (message: string) => {
      onStatus(message);
      onClearError();
    },
    [onClearError, onStatus],
  );

  const handleGridModifiersChange = useCallback((next: QueryGridModifiers) => {
    setQueryGridModifiers(next);
  }, []);

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
      void runQueryPage(activeQuerySql, 0, "filter/sort", {
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
    queryGridModifiers,
    queryResult?.isRowQuery,
    runQueryPage,
    selectedConnectionId,
  ]);

  const resetForDataReset = useCallback(() => {
    clearQueryHistoryStorage();
    clearFilterDebounceTimer();
    invalidateQueryRequests();
    setQueryPageSize(DEFAULT_QUERY_PAGE_SIZE);
    setQueryGridModifiers(DEFAULT_QUERY_GRID_MODIFIERS);
    appliedGridModifiersRef.current = DEFAULT_QUERY_GRID_MODIFIERS;
    setQueryHistory([]);
    setQueryResult(null);
    setActiveQuerySql(null);
    setCancelRequested(false);
  }, [clearFilterDebounceTimer, invalidateQueryRequests]);

  const setSqlSelection = useCallback((selection: SqlEditorSelection) => {
    sqlSelectionRef.current = selection;
  }, []);

  return {
    sqlText,
    setSqlText,
    queryResult,
    queryPageSize,
    queryGridModifiers,
    cancelRequested,
    queryHistoryForSelectedConnection,
    runQueryPage,
    runCurrentSql,
    handleNextPage,
    handlePreviousPage,
    handleQueryPageSizeChange,
    cancelRunningQuery,
    applyQueryHistory,
    runQueryHistory,
    clearSelectedConnectionHistory,
    applySqlTemplate,
    formatSql,
    copyResultPage,
    exportResultCsv,
    handleGridCopyFeedback,
    handleGridModifiersChange,
    resetQueryGridModifiers,
    resetForDataReset,
    setSqlSelection,
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

async function withPromiseTimeout<T>(
  promise: Promise<T>,
  timeoutMs: number,
  message: string,
): Promise<T> {
  let timeoutHandle: ReturnType<typeof window.setTimeout> | undefined;

  try {
    const timeoutPromise = new Promise<T>((_, reject) => {
      timeoutHandle = window.setTimeout(() => {
        reject(new Error(message));
      }, timeoutMs);
    });

    return await Promise.race([promise, timeoutPromise]);
  } finally {
    if (timeoutHandle) {
      window.clearTimeout(timeoutHandle);
    }
  }
}

function createQueryHistoryId(): string {
  if (typeof crypto !== "undefined" && "randomUUID" in crypto) {
    return crypto.randomUUID();
  }

  return `qh-${Date.now()}-${Math.random().toString(16).slice(2)}`;
}
