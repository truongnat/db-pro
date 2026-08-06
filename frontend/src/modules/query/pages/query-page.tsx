import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { format as formatSql } from "sql-formatter";

import { useConnectionStore } from "@/commons/stores/connection.store";
import { useTranslation } from "@/commons/locales/useTranslation";
import { Button } from "@/components/ui/button";
import { ExportDialog } from "@/modules/export/components/export-dialog";

import { ExplainPlanView } from "../components/explain-plan";
import { LocalHistoryPanel } from "../components/local-history-panel";
import { QueryEditor } from "../components/query-editor";
import { QueryHistoryPanel } from "../components/query-history-panel";
import { QueryTabs } from "../components/query-tabs";
import { QueryToolbar } from "../components/query-toolbar";
import { ResultGrid } from "../components/result-grid";
import { ResultTabs } from "../components/result-tabs";
import { RunConfigDialog } from "../components/run-config-dialog";
import { RunConfigList } from "../components/run-config-list";
import { SavedQueriesTree } from "../components/saved-queries-tree";
import { SqlFileOperations } from "../components/sql-file-operations";
import { TransactionBar } from "../components/transaction-bar";
import {
  useCancelQuery,
  useExecuteQuery,
  useExecuteQueryMulti,
  useExplainPlan,
  useQueryHistory,
} from "../queries/query.queries";
import { pushLocalHistory } from "../services/local-history";
import { debouncedSaveTabs, loadTabs } from "../services/tab-persistence";
import { useQueryModuleStore } from "../state/query.store";
import type { Row, RunConfig } from "../types/query.types";

export function QueryPage() {
  const { t } = useTranslation();
  const activeConnectionId = useConnectionStore((s) => s.activeConnectionId);

  const activeTab = useQueryModuleStore((s) =>
    s.tabs.find((t) => t.id === s.activeTabId),
  );
  const setSql = useQueryModuleStore((s) => s.setSql);
  const setStatus = useQueryModuleStore((s) => s.setStatus);
  const setResult = useQueryModuleStore((s) => s.setResult);
  const setExplainPlan = useQueryModuleStore((s) => s.setExplainPlan);
  const panelTab = useQueryModuleStore((s) => s.activeTab);
  const setPanelTab = useQueryModuleStore((s) => s.setActiveTab);
  const sort = activeTab?.sort ?? { column: null, direction: null };
  const setSort = useQueryModuleStore((s) => s.setSort);
  const historySearch = useQueryModuleStore((s) => s.historySearch);
  const setHistorySearch = useQueryModuleStore((s) => s.setHistorySearch);

  const sql = activeTab?.sql ?? "";
  const status = activeTab?.status ?? "idle";
  const error = activeTab?.error ?? null;
  const result = activeTab?.result ?? null;
  const explainPlan = activeTab?.explainPlan ?? null;

  const [exportOpen, setExportOpen] = useState(false);
  const [runConfigOpen, setRunConfigOpen] = useState(false);

  const executeMutation = useExecuteQuery();
  const executeMultiMutation = useExecuteQueryMulti();
  const explainMutation = useExplainPlan();
  const cancelMutation = useCancelQuery();
  const historyQuery = useQueryHistory(activeConnectionId ?? "");

  const restoredRef = useRef<string | null>(null);
  useEffect(() => {
    if (!activeConnectionId) return;
    if (restoredRef.current === activeConnectionId) return;
    restoredRef.current = activeConnectionId;
    const persisted = loadTabs(activeConnectionId);
    if (persisted && persisted.tabs.length > 0) {
      useQueryModuleStore.getState().restoreTabs(persisted.tabs, persisted.activeTabId);
    }
  }, [activeConnectionId]);

  useEffect(() => {
    if (!activeConnectionId) return;
    const unsub = useQueryModuleStore.subscribe((state) => {
      debouncedSaveTabs(activeConnectionId, state.tabs, state.activeTabId);
    });
    return unsub;
  }, [activeConnectionId]);

  const handleExecute = useCallback(() => {
    if (activeConnectionId && sql.trim()) {
      pushLocalHistory(sql.trim());
      executeMultiMutation.mutate({ connectionId: activeConnectionId, sql: sql.trim() });
    }
  }, [activeConnectionId, sql, executeMultiMutation]);

  const handleCancel = useCallback(() => {
    if (activeConnectionId) {
      cancelMutation.mutate({ connectionId: activeConnectionId });
    }
  }, [activeConnectionId, cancelMutation]);

  const handleExplain = useCallback(() => {
    if (activeConnectionId && sql.trim()) {
      explainMutation.mutate({ connectionId: activeConnectionId, sql: sql.trim() });
    }
  }, [activeConnectionId, sql, explainMutation]);

  const handleClear = useCallback(() => {
    setSql("");
  }, [setSql]);

  const handleFormat = useCallback(() => {
    if (!sql.trim()) return;
    try {
      const formatted = formatSql(sql, { language: "postgresql" });
      setSql(formatted);
    } catch {
      // leave SQL unchanged on format error
    }
  }, [sql, setSql]);

  const handleInsertTemplate = useCallback(
    (templateSql: string) => {
      setSql(sql.trim() ? `${sql}\n${templateSql}` : templateSql);
    },
    [sql, setSql],
  );

  const handleSort = useCallback(
    (column: string) => {
      if (sort.column === column) {
        if (sort.direction === "asc") {
          setSort({ column, direction: "desc" });
        } else if (sort.direction === "desc") {
          setSort({ column: null, direction: null });
        } else {
          setSort({ column, direction: "asc" });
        }
      } else {
        setSort({ column, direction: "asc" });
      }
    },
    [sort, setSort],
  );

  const handleSelectHistoryEntry = useCallback(
    (entrySql: string) => {
      setSql(entrySql);
      setPanelTab("results");
    },
    [setSql, setPanelTab],
  );

  const handleSelectSavedQuery = useCallback(
    (querySql: string) => {
      setSql(querySql);
    },
    [setSql],
  );

  const handleSelectRunConfig = useCallback(
    (config: RunConfig) => {
      setSql(config.sql);
    },
    [setSql],
  );

  const sortedRows = useMemo(() => {
    if (!result?.rows || !sort.column || !sort.direction) return result?.rows ?? [];

    const colIdx = result.columns.findIndex((c) => c.name === sort.column);
    if (colIdx === -1) return result.rows;

    const sorted = [...result.rows].sort((a, b) => {
      const aVal = getSortValue(a[colIdx]);
      const bVal = getSortValue(b[colIdx]);
      if (aVal < bVal) return -1;
      if (aVal > bVal) return 1;
      return 0;
    });

    return sort.direction === "desc" ? sorted.reverse() : sorted;
  }, [result, sort]);

  if (!activeConnectionId) {
    return (
      <div className="flex items-center justify-center py-12">
        <p className="text-muted-foreground">
          {t("query.connectFirst")}
        </p>
      </div>
    );
  }

  const tabs = [
    { id: "results" as const, label: t("query.results") },
    { id: "explain" as const, label: t("query.explain") },
    { id: "history" as const, label: t("query.history") },
    { id: "local-history" as const, label: t("query.localHistory") },
  ];

  return (
    <div className="flex h-full">
      <div
        className="flex flex-col border-r border-border"
        style={{
          width: "220px",
          minWidth: "180px",
        }}
      >
        <div className="border-b border-border p-2">
          <SqlFileOperations sql={sql} onSqlLoaded={setSql} />
        </div>
        <div className="flex-1 overflow-auto">
          <SavedQueriesTree
            connectionId={activeConnectionId}
            onSelectQuery={handleSelectSavedQuery}
          />
        </div>
        <div
          className="border-t border-border"
          style={{ maxHeight: "40%" }}
        >
          <RunConfigList
            connectionId={activeConnectionId}
            onSelect={handleSelectRunConfig}
            onNew={() => setRunConfigOpen(true)}
          />
        </div>
      </div>

      <div className="flex flex-1 flex-col overflow-hidden">
        <QueryToolbar
        onExecute={handleExecute}
        onCancel={handleCancel}
        onExplain={handleExplain}
        onClear={handleClear}
        onExport={() => setExportOpen(true)}
        onFormat={handleFormat}
        onInsertTemplate={handleInsertTemplate}
        isExecuting={executeMultiMutation.isPending}
        isExplaining={explainMutation.isPending}
        hasConnection={!!activeConnectionId}
        hasSql={!!sql.trim()}
      />

      <TransactionBar />

      <QueryTabs />

      <div className="h-[35%] min-h-[120px] border-b">
        <QueryEditor
          value={sql}
          onChange={setSql}
          onExecute={handleExecute}
        />
      </div>

      <div className="flex min-h-0 flex-1 flex-col">
        <div className="flex border-b border-border">
          {tabs.map((tab) => (
            <Button
              key={tab.id}
              type="button"
              variant="ghost"
              className={`px-4 py-2 text-sm transition-colors ${
                panelTab === tab.id ? "text-foreground" : "text-muted-foreground"
              } ${
                panelTab === tab.id
                  ? "border-b-2 border-primary"
                  : "border-b-2 border-transparent"
              }`}
              onClick={() => setPanelTab(tab.id)}
            >
              {tab.label}
            </Button>
          ))}
        </div>

        <div className="min-h-0 flex-1">
          {status === "error" && error && panelTab === "results" && (
            <div className="m-3 rounded-sm bg-destructive px-3 py-2 text-sm text-white">
              {error}
            </div>
          )}

          {panelTab === "results" && result && (
            <>
              <ResultTabs />
              <ResultGrid
              columns={result.columns}
              rows={sortedRows as Row[]}
              sort={sort}
              onSort={handleSort}
              durationMs={result.durationMs}
              rowCount={result.rowCount}
            />
            </>
          )}

          {panelTab === "results" && !result && status !== "error" && (
            <div className="flex items-center justify-center py-12">
              <p className="text-muted-foreground">
                {t("query.enterSql")}
              </p>
            </div>
          )}

          {panelTab === "explain" && explainPlan && (
            <ExplainPlanView plan={explainPlan} />
          )}

          {panelTab === "explain" && !explainPlan && (
            <div className="flex items-center justify-center py-12">
              <p className="text-muted-foreground">
                {t("query.noResults")}
              </p>
            </div>
          )}

          {panelTab === "history" && (
            <QueryHistoryPanel
              entries={historyQuery.data ?? []}
              search={historySearch}
              onSearchChange={setHistorySearch}
              onSelectEntry={handleSelectHistoryEntry}
              isLoading={historyQuery.isLoading}
            />
          )}

          {panelTab === "local-history" && (
            <LocalHistoryPanel onSelectEntry={handleSelectHistoryEntry} />
          )}
        </div>
      </div>
      </div>

      <ExportDialog
        open={exportOpen}
        onClose={() => setExportOpen(false)}
        connectionId={activeConnectionId}
        sql={sql.trim()}
      />

      <RunConfigDialog
        open={runConfigOpen}
        onClose={() => setRunConfigOpen(false)}
        connectionId={activeConnectionId}
        defaultSql={sql.trim()}
      />
    </div>
  );
}

function getSortValue(cell: { type: string; value?: unknown } | undefined): string | number {
  if (!cell || cell.type === "null") return "";
  if (cell.type === "int64" || cell.type === "float64") return cell.value as number;
  return String(cell.value ?? "");
}
