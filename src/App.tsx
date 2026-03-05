import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { loadNavigator, resetConnections, testConnection } from "@/lib/tauri";
import { ConnectionModal } from "@/features/connections/ConnectionModal";
import { ConnectionSidebar } from "@/features/connections/ConnectionSidebar";
import { clearConnectionPanelCache } from "@/features/connections/cache";
import { SAMPLE_CONNECTION_ID } from "@/features/connections/constants";
import { useConnectionController } from "@/features/connections/useConnectionController";
import { getSqlCompletionCatalog } from "@/features/sql-editor/completions";
import { starterSql } from "@/features/sql-editor/statement";
import {
  buildQualifiedObjectName,
  buildDdlTemplateSql,
  buildInsertTemplateSql,
  buildSelectFromObjectSql,
  buildUpdateTemplateSql,
} from "@/features/navigator/sql";
import type { NavigatorActionRequest } from "@/features/navigator/actions";
import { copyTextToClipboard } from "@/features/query/clipboard";
import { classifyQueryError, formatError } from "@/features/query/errors";
import { useQueryController } from "@/features/query/useQueryController";
import { QueryWorkbench } from "@/features/workbench/QueryWorkbench";
import type { NavigatorTree } from "@/types";

const NAVIGATOR_FRONTEND_TIMEOUT_MS = 15_000;

function App() {
  const [statusText, setStatusText] = useState(
    "Ready. Sample SQLite is available by default.",
  );
  const [lastErrorText, setLastErrorText] = useState<string | null>(null);
  const [busyAction, setBusyAction] = useState<string | null>(null);

  const [navigatorTree, setNavigatorTree] = useState<NavigatorTree | null>(null);
  const [navigatorLoading, setNavigatorLoading] = useState(false);
  const [navigatorError, setNavigatorError] = useState<string | null>(null);
  const navigatorRequestIdRef = useRef(0);

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

  const applyErrorStatus = useCallback((error: unknown) => {
    const classified = classifyQueryError(error);
    setStatusText(`${classified.headline}: ${classified.details}`);
    setLastErrorText(`${classified.details} ${classified.action}`);
  }, []);

  const clearErrorStatus = useCallback(() => {
    setLastErrorText(null);
  }, []);

  const {
    connections,
    selectedConnection,
    selectedConnectionId,
    setSelectedConnectionId,
    draft,
    setDraft,
    connectionModalOpen,
    connectionModalMode,
    openCreateModal,
    openEditModal,
    closeConnectionModal,
    resetConnectionDraft,
    submitConnectionForm,
    removeConnection,
    applyResetConnectionState,
  } = useConnectionController({
    onSetBusyAction: setBusyAction,
    onStatus: setStatusText,
    onClearError: clearErrorStatus,
    onError: applyErrorStatus,
  });

  const connectionPanelBusy = busyAction !== null;

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
    if (!selectedConnectionId) {
      setNavigatorTree(null);
      setNavigatorError(null);
      return;
    }
    void refreshNavigator(selectedConnectionId);
  }, [refreshNavigator, selectedConnectionId]);

  const {
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
  } = useQueryController({
    initialSqlText: starterSql,
    selectedConnectionId,
    selectedConnection,
    busyAction,
    onSetBusyAction: setBusyAction,
    onStatus: setStatusText,
    onClearError: clearErrorStatus,
    onError: applyErrorStatus,
    onRefreshNavigator: refreshNavigator,
  });

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
      resetForDataReset();
      applyResetConnectionState(items, nextSelectedId);

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
  }, [
    applyErrorStatus,
    clearErrorStatus,
    applyResetConnectionState,
    resetForDataReset,
    refreshNavigator,
  ]);

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
            gridModifiersOverride: {
              quickFilter: "",
              sortColumn: "",
              sortDirection: "asc",
            },
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
        void runCurrentSql();
        return;
      }

      if ((event.metaKey || event.ctrlKey) && event.shiftKey && event.key.toLowerCase() === "f") {
        event.preventDefault();
        void formatSql();
        return;
      }

      if (event.key === "Escape" && busyAction === "run-query") {
        event.preventDefault();
        void cancelRunningQuery();
      }
    };

    window.addEventListener("keydown", onGlobalKeyDown);
    return () => {
      window.removeEventListener("keydown", onGlobalKeyDown);
    };
  }, [busyAction, cancelRunningQuery, formatSql, runCurrentSql]);

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
          onOpenCreateModal={openCreateModal}
          onOpenEditModal={(connectionId) => {
            void openEditModal(connectionId);
          }}
          onResetData={() => {
            void handleResetData();
          }}
          onSelectConnection={setSelectedConnectionId}
          onDeleteConnection={(connectionId) => {
            void removeConnection(connectionId);
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
          onSqlSelectionChange={setSqlSelection}
          onQueryPageSizeChange={(nextPageSize) => {
            void handleQueryPageSizeChange(nextPageSize);
          }}
          onGridModifiersChange={handleGridModifiersChange}
          onApplyQueryHistory={applyQueryHistory}
          onRunQueryHistory={(historyId) => {
            void runQueryHistory(historyId);
          }}
          onClearQueryHistory={clearSelectedConnectionHistory}
          onApplySqlTemplate={applySqlTemplate}
          onFormatSql={formatSql}
          onCopyResultPage={(columns, rows) => {
            void copyResultPage(columns, rows);
          }}
          onExportResultCsv={(columns, rows) => {
            exportResultCsv(columns, rows);
          }}
          onGridCopyFeedback={handleGridCopyFeedback}
          onClearError={clearErrorStatus}
          onTestConnection={() => {
            void handleTestConnection();
          }}
          onCancelQuery={() => {
            void cancelRunningQuery();
          }}
          onRunQuery={() => {
            void runCurrentSql();
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
        onClose={closeConnectionModal}
        onSubmit={(event) => {
          void submitConnectionForm(event);
        }}
        onDraftChange={setDraft}
        onResetDraft={resetConnectionDraft}
      />
    </div>
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

export default App;
