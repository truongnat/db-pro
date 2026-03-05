import {
  type FormEvent,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";

import {
  cancelQuery,
  deleteConnection,
  executeQuery,
  getConnection,
  listConnections,
  loadNavigator,
  resetConnections,
  testConnection,
  upsertConnection,
} from "@/lib/tauri";
import { ConnectionModal } from "@/features/connections/ConnectionModal";
import { ConnectionSidebar } from "@/features/connections/ConnectionSidebar";
import {
  clearConnectionPanelCache,
  loadCachedDraft,
  loadCachedQueryPageSize,
  loadCachedSelectedConnectionId,
  resetCachedDraft,
  saveCachedDraft,
  saveCachedQueryPageSize,
  saveCachedSelectedConnectionId,
} from "@/features/connections/cache";
import { SAMPLE_CONNECTION_ID } from "@/features/connections/constants";
import {
  defaultConnectionDraft,
  fromConnectionInput,
  toConnectionInput,
  type ConnectionDraft,
} from "@/features/connections/draft";
import { getSqlCompletionCatalog } from "@/features/sql-editor/completions";
import { formatSqlText } from "@/features/sql-editor/format";
import { detectSqlTarget, starterSql } from "@/features/sql-editor/statement";
import type { SqlEditorSelection } from "@/features/sql-editor/types";
import {
  buildQualifiedObjectName,
  buildDdlTemplateSql,
  buildInsertTemplateSql,
  buildSelectFromObjectSql,
  buildUpdateTemplateSql,
} from "@/features/navigator/sql";
import type { NavigatorActionRequest } from "@/features/navigator/actions";
import { copyTextToClipboard, toTabDelimited } from "@/features/query/clipboard";
import {
  buildResultCsvFilename,
  downloadCsv,
  toCsv,
} from "@/features/query/csv";
import type { QueryGridModifiers } from "@/features/query/QueryResultGrid";
import {
  appendQueryHistory,
  clearQueryHistory,
  loadQueryHistory,
  saveQueryHistory,
  type QueryHistoryEntry,
} from "@/features/query/history";
import { QueryWorkbench } from "@/features/workbench/QueryWorkbench";
import type { ConnectionListItem, NavigatorTree, QueryResult } from "@/types";

const DEFAULT_QUERY_PAGE_SIZE = 500;
const NAVIGATOR_FRONTEND_TIMEOUT_MS = 15_000;
const QUERY_FRONTEND_TIMEOUT_MS = 45_000;
const FILTER_DEBOUNCE_MS = 300;
const DEFAULT_QUERY_GRID_MODIFIERS: QueryGridModifiers = {
  quickFilter: "",
  sortColumn: "",
  sortDirection: "asc",
};

function App() {
  const [connections, setConnections] = useState<ConnectionListItem[]>([]);
  const [selectedConnectionId, setSelectedConnectionId] = useState<string | null>(() =>
    loadCachedSelectedConnectionId(),
  );

  const [draft, setDraft] = useState<ConnectionDraft>(defaultConnectionDraft);
  const [connectionModalOpen, setConnectionModalOpen] = useState(false);
  const [connectionModalMode, setConnectionModalMode] = useState<"create" | "edit">(
    "create",
  );
  const [editingConnectionId, setEditingConnectionId] = useState<string | null>(null);
  const [editBaselineDraft, setEditBaselineDraft] = useState<ConnectionDraft | null>(null);

  const [sqlText, setSqlText] = useState(starterSql);
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
  const filterDebounceTimerRef = useRef<number | null>(null);
  const appliedGridModifiersRef = useRef<QueryGridModifiers>(
    DEFAULT_QUERY_GRID_MODIFIERS,
  );
  const sqlSelectionRef = useRef<SqlEditorSelection | null>(null);

  const [statusText, setStatusText] = useState(
    "Ready. Sample SQLite is available by default.",
  );
  const [lastErrorText, setLastErrorText] = useState<string | null>(null);
  const [busyAction, setBusyAction] = useState<string | null>(null);
  const [cancelRequested, setCancelRequested] = useState(false);

  const [navigatorTree, setNavigatorTree] = useState<NavigatorTree | null>(null);
  const [navigatorLoading, setNavigatorLoading] = useState(false);
  const [navigatorError, setNavigatorError] = useState<string | null>(null);
  const navigatorRequestIdRef = useRef(0);

  const selectedConnection = useMemo(
    () => connections.find((item) => item.id === selectedConnectionId) ?? null,
    [connections, selectedConnectionId],
  );

  const navigatorObjectCount = useMemo(() => {
    if (!navigatorTree) {
      return 0;
    }
    return navigatorTree.schemas.reduce(
      (acc, schema) => acc + schema.tables.length + schema.views.length,
      0,
    );
  }, [navigatorTree]);

  const sqlCompletionCatalog = useMemo(
    () => getSqlCompletionCatalog(navigatorTree),
    [navigatorTree],
  );

  const queryHistoryForSelectedConnection = useMemo(() => {
    if (!selectedConnectionId) {
      return [];
    }

    return queryHistory
      .filter((entry) => entry.connectionId === selectedConnectionId)
      .slice(0, 50);
  }, [queryHistory, selectedConnectionId]);

  const connectionPanelBusy = busyAction !== null;

  const applyErrorStatus = useCallback((error: unknown) => {
    const message = formatError(error);
    setStatusText(message);
    setLastErrorText(message);
  }, []);

  const clearErrorStatus = useCallback(() => {
    setLastErrorText(null);
  }, []);

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

  const refreshConnections = useCallback(async (preferredId?: string) => {
    try {
      const items = await listConnections();
      setConnections(items);
      clearErrorStatus();

      setSelectedConnectionId((previous) => {
        if (preferredId && items.some((item) => item.id === preferredId)) {
          return preferredId;
        }
        if (previous && items.some((item) => item.id === previous)) {
          return previous;
        }
        return items[0]?.id ?? null;
      });
    } catch (error) {
      setConnections([]);
      setSelectedConnectionId(null);
      applyErrorStatus(error);
    }
  }, [applyErrorStatus, clearErrorStatus]);

  const refreshNavigator = useCallback(
    async (connectionId?: string) => {
      const targetConnectionId = connectionId ?? selectedConnectionId;
      if (!targetConnectionId) {
        navigatorRequestIdRef.current += 1;
        setNavigatorTree(null);
        setNavigatorError(null);
        setNavigatorLoading(false);
        return;
      }

      const requestId = navigatorRequestIdRef.current + 1;
      navigatorRequestIdRef.current = requestId;

      try {
        setNavigatorLoading(true);
        setNavigatorError(null);
        const tree = await withPromiseTimeout(
          loadNavigator(targetConnectionId),
          NAVIGATOR_FRONTEND_TIMEOUT_MS,
          `Schema navigator load timed out after ${NAVIGATOR_FRONTEND_TIMEOUT_MS} ms`,
        );

        if (navigatorRequestIdRef.current !== requestId) {
          return;
        }

        setNavigatorTree(tree);
        clearErrorStatus();
      } catch (error) {
        if (navigatorRequestIdRef.current !== requestId) {
          return;
        }

        setNavigatorTree(null);
        const message = formatError(error);
        setNavigatorError(message);
        setLastErrorText(message);
      } finally {
        if (navigatorRequestIdRef.current === requestId) {
          setNavigatorLoading(false);
        }
      }
    },
    [clearErrorStatus, selectedConnectionId],
  );

  useEffect(() => {
    void refreshConnections();
  }, [refreshConnections]);

  useEffect(() => {
    if (!selectedConnectionId) {
      setNavigatorTree(null);
      setNavigatorError(null);
      return;
    }
    void refreshNavigator(selectedConnectionId);
  }, [refreshNavigator, selectedConnectionId]);

  useEffect(() => {
    return () => {
      if (filterDebounceTimerRef.current !== null) {
        window.clearTimeout(filterDebounceTimerRef.current);
      }
    };
  }, []);

  useEffect(() => {
    clearFilterDebounceTimer();
    setQueryGridModifiers(DEFAULT_QUERY_GRID_MODIFIERS);
    appliedGridModifiersRef.current = DEFAULT_QUERY_GRID_MODIFIERS;
    setQueryResult(null);
    setActiveQuerySql(null);
  }, [clearFilterDebounceTimer, selectedConnectionId]);

  useEffect(() => {
    saveCachedSelectedConnectionId(selectedConnectionId);
  }, [selectedConnectionId]);

  useEffect(() => {
    saveCachedQueryPageSize(queryPageSize);
  }, [queryPageSize]);

  useEffect(() => {
    saveQueryHistory(queryHistory);
  }, [queryHistory]);

  useEffect(() => {
    if (connectionModalMode === "create") {
      saveCachedDraft(draft);
    }
  }, [connectionModalMode, draft]);

  const handleOpenCreateModal = useCallback(() => {
    setConnectionModalMode("create");
    setEditingConnectionId(null);
    setEditBaselineDraft(null);
    setDraft(loadCachedDraft(defaultConnectionDraft));
    setConnectionModalOpen(true);
  }, []);

  const handleOpenEditModal = useCallback(async (connectionId: string) => {
    try {
      setBusyAction("load-connection");
      const input = await getConnection(connectionId);
      const nextDraft = fromConnectionInput(input);

      setDraft(nextDraft);
      setEditBaselineDraft(nextDraft);
      setEditingConnectionId(connectionId);
      setConnectionModalMode("edit");
      setConnectionModalOpen(true);
      setStatusText(`Editing connection '${input.name}'.`);
      clearErrorStatus();
    } catch (error) {
      applyErrorStatus(error);
    } finally {
      setBusyAction(null);
    }
  }, [applyErrorStatus, clearErrorStatus]);

  const handleCloseConnectionModal = useCallback(() => {
    setConnectionModalOpen(false);
  }, []);

  const handleResetConnectionDraft = useCallback(() => {
    if (connectionModalMode === "edit" && editBaselineDraft) {
      setDraft(editBaselineDraft);
      return;
    }

    setDraft(defaultConnectionDraft);
    resetCachedDraft();
  }, [connectionModalMode, editBaselineDraft]);

  const handleSaveConnection = useCallback(
    async (event: FormEvent<HTMLFormElement>) => {
      event.preventDefault();

      try {
        setBusyAction("save-connection");

        if (connectionModalMode === "edit" && !editingConnectionId) {
          throw new Error("Missing connection id for edit mode.");
        }

        const connectionId =
          connectionModalMode === "edit" ? editingConnectionId ?? undefined : undefined;
        const saved = await upsertConnection(toConnectionInput(draft, connectionId));

        setStatusText(
          connectionModalMode === "create"
            ? `Saved connection '${saved.name}' (persisted).`
            : `Updated connection '${saved.name}'.`,
        );
        clearErrorStatus();

        await refreshConnections(saved.id);
        setConnectionModalOpen(false);
        setConnectionModalMode("create");
        setEditingConnectionId(null);
        setEditBaselineDraft(null);

        if (connectionModalMode === "create") {
          setDraft(defaultConnectionDraft);
          resetCachedDraft();
        }
      } catch (error) {
        applyErrorStatus(error);
      } finally {
        setBusyAction(null);
      }
    },
    [
      applyErrorStatus,
      clearErrorStatus,
      connectionModalMode,
      draft,
      editingConnectionId,
      refreshConnections,
    ],
  );

  const handleRemoveConnection = useCallback(
    async (connectionId: string) => {
      const item = connections.find((entry) => entry.id === connectionId);
      const confirmed = window.confirm(
        `Delete connection '${item?.name ?? connectionId}'? This action cannot be undone.`,
      );
      if (!confirmed) {
        return;
      }

      try {
        setBusyAction("delete-connection");
        await deleteConnection(connectionId);
        setStatusText("Connection removed.");
        clearErrorStatus();
        await refreshConnections();
      } catch (error) {
        applyErrorStatus(error);
      } finally {
        setBusyAction(null);
      }
    },
    [applyErrorStatus, clearErrorStatus, connections, refreshConnections],
  );

  const handleResetData = useCallback(async () => {
    const confirmed = window.confirm(
      "Reset all connections and cached panel data? Sample SQLite will be recreated.",
    );
    if (!confirmed) {
      return;
    }

    try {
      setBusyAction("reset-data");

      const items = await resetConnections();
      const nextSelectedId =
        items.find((item) => item.id === SAMPLE_CONNECTION_ID)?.id ?? items[0]?.id ?? null;

      clearConnectionPanelCache();
      clearQueryHistory();
      clearFilterDebounceTimer();
      setConnections(items);
      setSelectedConnectionId(nextSelectedId);
      setQueryPageSize(DEFAULT_QUERY_PAGE_SIZE);
      setQueryGridModifiers(DEFAULT_QUERY_GRID_MODIFIERS);
      appliedGridModifiersRef.current = DEFAULT_QUERY_GRID_MODIFIERS;
      setQueryHistory([]);

      setConnectionModalOpen(false);
      setConnectionModalMode("create");
      setEditingConnectionId(null);
      setEditBaselineDraft(null);
      setDraft(defaultConnectionDraft);

      if (nextSelectedId) {
        await refreshNavigator(nextSelectedId);
      } else {
        setNavigatorTree(null);
        setNavigatorError(null);
      }

      setStatusText("Data reset completed. Sample SQLite is restored.");
      clearErrorStatus();
    } catch (error) {
      applyErrorStatus(error);
    } finally {
      setBusyAction(null);
    }
  }, [applyErrorStatus, clearErrorStatus, clearFilterDebounceTimer, refreshNavigator]);

  const handleTestConnection = useCallback(async () => {
    if (!selectedConnectionId) {
      setStatusText("Select a connection first.");
      return;
    }

    try {
      setBusyAction("test-connection");
      const result = await testConnection(selectedConnectionId);
      setStatusText(result);
      clearErrorStatus();
    } catch (error) {
      applyErrorStatus(error);
    } finally {
      setBusyAction(null);
    }
  }, [applyErrorStatus, clearErrorStatus, selectedConnectionId]);

  type RunQueryPageOptions = {
    pageSizeOverride?: number;
    gridModifiersOverride?: QueryGridModifiers;
    skipHistory?: boolean;
  };

  const runQueryPage = useCallback(
    async (
      sql: string,
      offset: number,
      runSource: string,
      options?: RunQueryPageOptions,
    ) => {
      if (!selectedConnectionId) {
        setStatusText("Select a connection first.");
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

      try {
        setCancelRequested(false);
        setBusyAction("run-query");
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

        setStatusText(`${result.message} in ${result.executionMs} ms (${runSource}).`);
        clearErrorStatus();
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
          void refreshNavigator(selectedConnectionId);
        }
      } catch (error) {
        if (isTimeoutError(error)) {
          void cancelQuery(selectedConnectionId).catch(() => undefined);
        }
        applyErrorStatus(error);
      } finally {
        setBusyAction(null);
        setCancelRequested(false);
      }
    },
    [
      applyErrorStatus,
      clearErrorStatus,
      queryPageSize,
      queryGridModifiers,
      queryResult?.columns,
      refreshNavigator,
      selectedConnection,
      selectedConnectionId,
    ],
  );

  const handleRunQuery = useCallback(async () => {
    if (!selectedConnectionId) {
      setStatusText("Select a connection first.");
      return;
    }

    const sqlTarget = detectSqlTarget(sqlText, sqlSelectionRef.current);
    if (!sqlTarget.sql.trim()) {
      setStatusText("SQL is empty.");
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
  }, [resetQueryGridModifiers, runQueryPage, selectedConnectionId, sqlText]);

  const handleNextPage = useCallback(async () => {
    if (!selectedConnectionId) {
      setStatusText("Select a connection first.");
      return;
    }

    if (!queryResult?.isRowQuery || !activeQuerySql) {
      setStatusText("Run a row query first.");
      return;
    }

    if (!queryResult.hasMore) {
      setStatusText("No more rows.");
      return;
    }

    const nextOffset = queryResult.pageOffset + queryResult.pageSize;
    await runQueryPage(activeQuerySql, nextOffset, "next page");
  }, [activeQuerySql, queryResult, runQueryPage, selectedConnectionId]);

  const handlePreviousPage = useCallback(async () => {
    if (!selectedConnectionId) {
      setStatusText("Select a connection first.");
      return;
    }

    if (!queryResult?.isRowQuery || !activeQuerySql) {
      setStatusText("Run a row query first.");
      return;
    }

    if (queryResult.pageOffset === 0) {
      setStatusText("Already at the first page.");
      return;
    }

    const previousOffset = Math.max(queryResult.pageOffset - queryResult.pageSize, 0);
    await runQueryPage(activeQuerySql, previousOffset, "previous page");
  }, [activeQuerySql, queryResult, runQueryPage, selectedConnectionId]);

  const handleQueryPageSizeChange = useCallback(
    async (nextPageSize: number) => {
      if (!Number.isFinite(nextPageSize) || nextPageSize <= 0) {
        return;
      }
      if (nextPageSize === queryPageSize) {
        return;
      }

      setQueryPageSize(nextPageSize);
      setStatusText(`Query page size set to ${nextPageSize}.`);

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
    [activeQuerySql, queryPageSize, queryResult, runQueryPage, selectedConnectionId],
  );

  const handleCancelQuery = useCallback(async () => {
    if (!selectedConnectionId) {
      setStatusText("Select a connection first.");
      return;
    }

    if (busyAction !== "run-query") {
      setStatusText("No running query to cancel.");
      return;
    }

    try {
      setCancelRequested(true);
      const result = await cancelQuery(selectedConnectionId);
      setStatusText(result);
      clearErrorStatus();
    } catch (error) {
      applyErrorStatus(error);
      setCancelRequested(false);
    }
  }, [applyErrorStatus, busyAction, clearErrorStatus, selectedConnectionId]);

  const handleApplyQueryHistory = useCallback(
    (historyId: string) => {
      const item = queryHistoryForSelectedConnection.find(
        (entry) => entry.id === historyId,
      );
      if (!item) {
        setStatusText("History entry not found.");
        return;
      }

      setSqlText(item.sql);
      setStatusText(
        `Loaded query from history (${item.runSource}, ${item.executionMs} ms).`,
      );
      clearErrorStatus();
    },
    [clearErrorStatus, queryHistoryForSelectedConnection],
  );

  const handleRunQueryHistory = useCallback(
    async (historyId: string) => {
      const item = queryHistoryForSelectedConnection.find(
        (entry) => entry.id === historyId,
      );
      if (!item) {
        setStatusText("History entry not found.");
        return;
      }

      setSqlText(item.sql);
      resetQueryGridModifiers();
      await runQueryPage(item.sql, 0, "history", {
        gridModifiersOverride: DEFAULT_QUERY_GRID_MODIFIERS,
      });
      clearErrorStatus();
    },
    [
      clearErrorStatus,
      queryHistoryForSelectedConnection,
      resetQueryGridModifiers,
      runQueryPage,
    ],
  );

  const handleClearQueryHistory = useCallback(() => {
    if (!selectedConnectionId) {
      setStatusText("Select a connection first.");
      return;
    }

    setQueryHistory((previous) =>
      previous.filter((entry) => entry.connectionId !== selectedConnectionId),
    );
    setStatusText("Cleared query history for current connection.");
  }, [selectedConnectionId]);

  const handleApplySqlTemplate = useCallback(
    (sql: string, label: string) => {
      const nextText = sqlText.trim() ? `${sqlText.trimEnd()}\n\n${sql}` : sql;
      setSqlText(nextText);
      setStatusText(`Inserted SQL template: ${label}.`);
      clearErrorStatus();
    },
    [clearErrorStatus, sqlText],
  );

  const handleFormatSql = useCallback(async () => {
    const nextSql = await formatSqlText(sqlText, selectedConnection?.engine);
    if (nextSql === sqlText) {
      setStatusText("SQL is already formatted.");
      clearErrorStatus();
      return;
    }

    setSqlText(nextSql);
    setStatusText("SQL formatted.");
    clearErrorStatus();
  }, [clearErrorStatus, selectedConnection?.engine, sqlText]);

  const handleCopyResultPage = useCallback(async (columns: string[], rows: string[][]) => {
    if (columns.length === 0) {
      setStatusText("No result rows to copy.");
      return;
    }

    try {
      const payload = toTabDelimited(columns, rows, true);
      await copyTextToClipboard(payload);
      setStatusText(`Copied ${rows.length} row(s) from current page.`);
      clearErrorStatus();
    } catch (error) {
      applyErrorStatus(error);
    }
  }, [applyErrorStatus, clearErrorStatus]);

  const handleExportResultCsv = useCallback((columns: string[], rows: string[][]) => {
    if (columns.length === 0) {
      setStatusText("No result rows to export.");
      return;
    }

    try {
      const csv = toCsv(columns, rows);
      const filename = buildResultCsvFilename(selectedConnection?.name);
      downloadCsv(filename, csv);
      setStatusText(`Exported ${rows.length} row(s) to ${filename}.`);
      clearErrorStatus();
    } catch (error) {
      applyErrorStatus(error);
    }
  }, [applyErrorStatus, clearErrorStatus, selectedConnection?.name]);

  const handleGridCopyFeedback = useCallback(
    (message: string) => {
      setStatusText(message);
      clearErrorStatus();
    },
    [clearErrorStatus],
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

  const insertNavigatorSql = useCallback(
    (sql: string, message: string) => {
      setSqlText((previous) => (previous.trim() ? `${previous.trimEnd()}\n\n${sql}` : sql));
      setStatusText(message);
      clearErrorStatus();
    },
    [clearErrorStatus],
  );

  const handleNavigatorAction = useCallback(
    async (request: NavigatorActionRequest) => {
      const { schemaName, object, action } = request;
      const engine = selectedConnection?.engine;
      const qualifiedName = buildQualifiedObjectName(engine, schemaName, object.name);

      try {
        if (action === "copy_name") {
          await copyTextToClipboard(qualifiedName);
          setStatusText(`Copied object name ${qualifiedName}.`);
          clearErrorStatus();
          return;
        }

        if (action === "generate_select" || action === "open_data") {
          const selectSql = buildSelectFromObjectSql(
            engine,
            schemaName,
            object.name,
            queryPageSize,
          );

          if (action === "generate_select") {
            insertNavigatorSql(selectSql, `Generated SELECT for ${qualifiedName}.`);
            return;
          }

          setSqlText(selectSql);
          setStatusText(`Running SELECT for ${qualifiedName}...`);
          clearErrorStatus();
          resetQueryGridModifiers();
          await runQueryPage(selectSql, 0, "navigator", {
            gridModifiersOverride: DEFAULT_QUERY_GRID_MODIFIERS,
          });
          return;
        }

        if (action === "generate_insert") {
          const insertSql = buildInsertTemplateSql(engine, schemaName, object);
          insertNavigatorSql(insertSql, `Generated INSERT for ${qualifiedName}.`);
          return;
        }

        if (action === "generate_update") {
          const updateSql = buildUpdateTemplateSql(engine, schemaName, object);
          insertNavigatorSql(updateSql, `Generated UPDATE for ${qualifiedName}.`);
          return;
        }

        if (action === "generate_ddl") {
          const ddlSql = buildDdlTemplateSql(engine, schemaName, object);
          insertNavigatorSql(ddlSql, `Generated DDL template for ${qualifiedName}.`);
        }
      } catch (error) {
        applyErrorStatus(error);
      }
    },
    [
      applyErrorStatus,
      clearErrorStatus,
      insertNavigatorSql,
      queryPageSize,
      resetQueryGridModifiers,
      runQueryPage,
      selectedConnection?.engine,
    ],
  );

  useEffect(() => {
    const onGlobalKeyDown = (event: KeyboardEvent) => {
      if (event.defaultPrevented) {
        return;
      }

      if (event.key === "F5") {
        event.preventDefault();
        void handleRunQuery();
        return;
      }

      if ((event.metaKey || event.ctrlKey) && event.shiftKey && event.key.toLowerCase() === "f") {
        event.preventDefault();
        void handleFormatSql();
        return;
      }

      if (event.key === "Escape" && busyAction === "run-query") {
        event.preventDefault();
        void handleCancelQuery();
      }
    };

    window.addEventListener("keydown", onGlobalKeyDown);
    return () => {
      window.removeEventListener("keydown", onGlobalKeyDown);
    };
  }, [busyAction, handleCancelQuery, handleFormatSql, handleRunQuery]);

  return (
    <div className="relative min-h-screen overflow-hidden p-4 text-foreground">
      <div className="pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_20%_0%,rgba(59,130,246,0.12),transparent_45%),radial-gradient(circle_at_100%_100%,rgba(20,184,166,0.1),transparent_40%)]" />

      <div className="relative grid h-[calc(100vh-2rem)] grid-cols-1 gap-4 xl:grid-cols-[350px_1fr]">
        <ConnectionSidebar
          busy={connectionPanelBusy}
          resettingData={busyAction === "reset-data"}
          connections={connections}
          selectedConnectionId={selectedConnectionId}
          selectedConnection={selectedConnection}
          navigatorTree={navigatorTree}
          navigatorLoading={navigatorLoading}
          navigatorError={navigatorError}
          navigatorObjectCount={navigatorObjectCount}
          onOpenCreateModal={handleOpenCreateModal}
          onOpenEditModal={(connectionId) => {
            void handleOpenEditModal(connectionId);
          }}
          onResetData={() => {
            void handleResetData();
          }}
          onSelectConnection={setSelectedConnectionId}
          onDeleteConnection={(connectionId) => {
            void handleRemoveConnection(connectionId);
          }}
          onRefreshNavigator={() => {
            void refreshNavigator();
          }}
          onNavigatorAction={(request) => {
            void handleNavigatorAction(request);
          }}
        />

        <QueryWorkbench
          selectedConnection={selectedConnection}
          statusText={statusText}
          errorText={lastErrorText}
          busyAction={busyAction}
          cancelRequested={cancelRequested}
          sqlText={sqlText}
          queryResult={queryResult}
          queryHistory={queryHistoryForSelectedConnection}
          queryPageSize={queryPageSize}
          gridModifiers={queryGridModifiers}
          sqlEngine={selectedConnection?.engine}
          sqlCompletionCatalog={sqlCompletionCatalog}
          onSqlTextChange={setSqlText}
          onSqlSelectionChange={(selection) => {
            sqlSelectionRef.current = selection;
          }}
          onQueryPageSizeChange={(nextPageSize) => {
            void handleQueryPageSizeChange(nextPageSize);
          }}
          onGridModifiersChange={handleGridModifiersChange}
          onApplyQueryHistory={handleApplyQueryHistory}
          onRunQueryHistory={(historyId) => {
            void handleRunQueryHistory(historyId);
          }}
          onClearQueryHistory={handleClearQueryHistory}
          onApplySqlTemplate={handleApplySqlTemplate}
          onFormatSql={handleFormatSql}
          onCopyResultPage={(columns, rows) => {
            void handleCopyResultPage(columns, rows);
          }}
          onExportResultCsv={(columns, rows) => {
            handleExportResultCsv(columns, rows);
          }}
          onGridCopyFeedback={handleGridCopyFeedback}
          onClearError={clearErrorStatus}
          onTestConnection={() => {
            void handleTestConnection();
          }}
          onCancelQuery={() => {
            void handleCancelQuery();
          }}
          onRunQuery={() => {
            void handleRunQuery();
          }}
          onPreviousPage={() => {
            void handlePreviousPage();
          }}
          onNextPage={() => {
            void handleNextPage();
          }}
        />
      </div>

      <ConnectionModal
        open={connectionModalOpen}
        mode={connectionModalMode}
        draft={draft}
        saving={busyAction === "save-connection"}
        onClose={handleCloseConnectionModal}
        onSubmit={(event) => {
          void handleSaveConnection(event);
        }}
        onDraftChange={setDraft}
        onResetDraft={handleResetConnectionDraft}
      />
    </div>
  );
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

function formatError(error: unknown): string {
  if (typeof error === "string") {
    return error;
  }
  if (error instanceof Error) {
    return error.message;
  }
  return "Unexpected error";
}

function isTimeoutError(error: unknown): boolean {
  const message = formatError(error).toLowerCase();
  return message.includes("timed out") || message.includes("timeout");
}

export default App;
