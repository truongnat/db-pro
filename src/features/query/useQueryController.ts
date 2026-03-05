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
import type { ConnectionListItem, QueryResult } from "@/types";

import { copyTextToClipboard, toTabDelimited } from "./clipboard";
import type { QueryRunPageOptions, RunQueryPageFn } from "./controller-types";
import { buildResultCsvFilename, downloadCsv, toCsv } from "./csv";
import { isTimeoutError } from "./errors";
import {
  appendQueryHistory,
  clearQueryHistory as clearQueryHistoryStorage,
  loadQueryHistory,
  saveQueryHistory,
  type QueryHistoryEntry,
} from "./history";
import { useResultGridController } from "./useResultGridController";
import { detectSqlTarget } from "../sql-editor/statement";
import { formatSqlText } from "../sql-editor/format";
import type { SqlEditorSelection } from "../sql-editor/types";

const QUERY_FRONTEND_TIMEOUT_MS = 45_000;

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
  const [cancelRequested, setCancelRequested] = useState(false);

  const sqlSelectionRef = useRef<SqlEditorSelection | null>(null);
  const queryRequestIdRef = useRef(0);
  const runQueryPageRef = useRef<RunQueryPageFn>(async () => undefined);

  const queryHistoryForSelectedConnection = useMemo(() => {
    if (!selectedConnectionId) {
      return [];
    }

    return queryHistory
      .filter((entry) => entry.connectionId === selectedConnectionId)
      .slice(0, 50);
  }, [queryHistory, selectedConnectionId]);

  const invalidateQueryRequests = useCallback(() => {
    queryRequestIdRef.current += 1;
  }, []);

  const {
    queryPageSize,
    queryGridModifiers,
    handleGridModifiersChange,
    resetQueryGridModifiers,
    handleNextPage,
    handlePreviousPage,
    handleQueryPageSizeChange,
    markGridModifiersApplied,
    resetForDataReset: resetResultGridForDataReset,
  } = useResultGridController({
    selectedConnectionId,
    busyAction,
    activeQuerySql,
    queryResult,
    onStatus,
    onRunQueryPage: (sql, offset, runSource, options) =>
      runQueryPageRef.current(sql, offset, runSource, options),
  });

  useEffect(() => {
    invalidateQueryRequests();
    setQueryResult(null);
    setActiveQuerySql(null);
    onSetBusyAction((previous) => (previous === "run-query" ? null : previous));
    setCancelRequested(false);
  }, [invalidateQueryRequests, onSetBusyAction, selectedConnectionId]);

  useEffect(() => {
    saveQueryHistory(queryHistory);
  }, [queryHistory]);

  const runQueryPage = useCallback(
    async (
      sql: string,
      offset: number,
      runSource: string,
      options?: QueryRunPageOptions,
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
        markGridModifiersApplied(
          result.isRowQuery
            ? effectiveGridModifiers
            : {
                quickFilter: "",
                sortColumn: "",
                sortDirection: "asc",
              },
        );

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
  runQueryPageRef.current = runQueryPage;

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
      gridModifiersOverride: {
        quickFilter: "",
        sortColumn: "",
        sortDirection: "asc",
      },
    });
  }, [onStatus, resetQueryGridModifiers, runQueryPage, selectedConnectionId, sqlText]);

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
        gridModifiersOverride: {
          quickFilter: "",
          sortColumn: "",
          sortDirection: "asc",
        },
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

  const resetForDataReset = useCallback(() => {
    clearQueryHistoryStorage();
    invalidateQueryRequests();
    resetResultGridForDataReset();
    setQueryHistory([]);
    setQueryResult(null);
    setActiveQuerySql(null);
    setCancelRequested(false);
  }, [invalidateQueryRequests, resetResultGridForDataReset]);

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
