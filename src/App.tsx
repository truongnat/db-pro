import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { resetConnections, testConnection } from "@/lib/tauri";
import { ConnectionModal } from "@/features/connections/ConnectionModal";
import { ConnectionSidebar } from "@/features/connections/ConnectionSidebar";
import { clearConnectionPanelCache } from "@/features/connections/cache";
import { SAMPLE_CONNECTION_ID } from "@/features/connections/constants";
import { useConnectionController } from "@/features/connections/useConnectionController";
import { useNavigatorController } from "@/features/navigator/useNavigatorController";
import { getSqlCompletionCatalog } from "@/features/sql-editor/completions";
import { starterSql } from "@/features/sql-editor/statement";
import { classifyQueryError } from "@/features/query/errors";
import { useQueryController } from "@/features/query/useQueryController";
import { QueryWorkbench } from "@/features/workbench/QueryWorkbench";

function App() {
  const [statusText, setStatusText] = useState(
    "Ready. Sample SQLite is available by default.",
  );
  const [lastErrorText, setLastErrorText] = useState<string | null>(null);
  const [busyAction, setBusyAction] = useState<string | null>(null);

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
  const refreshNavigatorRef = useRef<(connectionId?: string) => void | Promise<void>>(
    () => undefined,
  );

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
    onRefreshNavigator: (connectionId) => refreshNavigatorRef.current(connectionId),
  });

  const {
    navigatorTree,
    navigatorLoading,
    navigatorError,
    navigatorObjectCount,
    refreshNavigator,
    handleNavigatorAction,
  } = useNavigatorController({
    selectedConnectionId,
    selectedConnectionEngine: selectedConnection?.engine,
    queryPageSize,
    onSetSqlText: setSqlText,
    onRunQueryPage: runQueryPage,
    onResetQueryGridModifiers: resetQueryGridModifiers,
    onStatus: setStatusText,
    onClearError: clearErrorStatus,
    onError: applyErrorStatus,
  });
  refreshNavigatorRef.current = refreshNavigator;

  const sqlCompletionCatalog = useMemo(
    () => getSqlCompletionCatalog(navigatorTree),
    [navigatorTree],
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
      resetForDataReset();
      applyResetConnectionState(items, nextSelectedId);

      if (nextSelectedId) {
        await refreshNavigator(nextSelectedId);
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

export default App;
