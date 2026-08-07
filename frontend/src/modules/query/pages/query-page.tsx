import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { format as formatSql } from "sql-formatter";

import { ResizableDock } from "@/commons/components/resizable-dock";
import { WorkspaceTabBar } from "@/commons/components/workspace-tab-bar";
import { onQueryAction } from "@/commons/commands/query-dispatch";
import { useConnectionStore } from "@/commons/stores/connection.store";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import type { ResultPanelTab } from "@/commons/types/workspace.types";
import { useTranslation } from "@/commons/locales/useTranslation";
import { createQueryTab } from "@/commons/factories/tab-factories";
import { Button } from "@/components/ui/button";
import { ExportDialog } from "@/modules/export/components/export-dialog";

import { ExplainPlanView } from "../components/explain-plan";
import { LocalHistoryPanel } from "../components/local-history-panel";
import { QueryEditor } from "../components/query-editor";
import { QueryHistoryPanel } from "../components/query-history-panel";
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
import {
  setTabSql,
  setTabSort,
} from "../controllers/query-workspace.controller";
import type { Row, RunConfig } from "../types/query.types";

export function QueryPage() {
  const { t } = useTranslation();
  const activeConnectionId = useConnectionStore((s) => s.activeConnectionId);

  const activeTab = useWorkspaceStore((s) => {
    const tab = s.tabs.find((t) => t.id === s.activeTabId);
    return tab?.kind === "query" ? tab : undefined;
  });
  const activeTabId = useWorkspaceStore((s) => s.activeTabId);

  const [panelTab, setPanelTab] = useState<ResultPanelTab>("results");
  const [historySearch, setHistorySearch] = useState("");

  const tabData = activeTab?.data;
  const sql = tabData?.sql ?? "";
  const status = tabData?.status ?? "idle";
  const error = tabData?.error ?? null;
  const result = tabData?.result ?? null;
  const explainPlan = tabData?.explainPlan ?? null;
  const sort = tabData?.sort ?? { column: null, direction: null };

  const [exportOpen, setExportOpen] = useState(false);
  const [runConfigOpen, setRunConfigOpen] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const executeMutation = useExecuteQuery();
  const executeMultiMutation = useExecuteQueryMulti();
  const explainMutation = useExplainPlan();
  const cancelMutation = useCancelQuery();
  const historyQuery = useQueryHistory(activeConnectionId ?? "");

  const initializedRef = useRef<string | null>(null);
  useEffect(() => {
    if (!activeConnectionId) return;
    if (initializedRef.current === activeConnectionId) return;
    initializedRef.current = activeConnectionId;

    const { tabs } = useWorkspaceStore.getState();
    const hasTabsForConnection = tabs.some(
      (t) => t.kind === "query" && t.connectionId === activeConnectionId,
    );
    if (!hasTabsForConnection) {
      const tab = createQueryTab(activeConnectionId);
      useWorkspaceStore.getState().openTab(tab);
    }
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
    if (activeTabId) setTabSql(activeTabId, "");
  }, [activeTabId]);

  const handleFormat = useCallback(() => {
    if (!activeTabId || !sql.trim()) return;
    try {
      const formatted = formatSql(sql, { language: "postgresql" });
      setTabSql(activeTabId, formatted);
    } catch {
      // leave SQL unchanged on format error
    }
  }, [activeTabId, sql, setTabSql]);

  const handleInsertTemplate = useCallback(
    (templateSql: string) => {
      if (!activeTabId) return;
      setTabSql(activeTabId, sql.trim() ? `${sql}\n${templateSql}` : templateSql);
    },
    [activeTabId, sql, setTabSql],
  );

  const handleSort = useCallback(
    (column: string) => {
      if (!activeTabId) return;
      if (sort.column === column) {
        if (sort.direction === "asc") {
          setTabSort(activeTabId, { column, direction: "desc" });
        } else if (sort.direction === "desc") {
          setTabSort(activeTabId, { column: null, direction: null });
        } else {
          setTabSort(activeTabId, { column, direction: "asc" });
        }
      } else {
        setTabSort(activeTabId, { column, direction: "asc" });
      }
    },
    [activeTabId, sort, setTabSort],
  );

  const handleSelectHistoryEntry = useCallback(
    (entrySql: string) => {
      if (!activeTabId) return;
      setTabSql(activeTabId, entrySql);
      setPanelTab("results");
    },
    [activeTabId, setTabSql],
  );

  const handleSelectSavedQuery = useCallback(
    (querySql: string) => {
      if (!activeTabId) return;
      setTabSql(activeTabId, querySql);
    },
    [activeTabId, setTabSql],
  );

  const handleSelectRunConfig = useCallback(
    (config: RunConfig) => {
      if (!activeTabId) return;
      setTabSql(activeTabId, config.sql);
    },
    [activeTabId, setTabSql],
  );

  const handleFileImport = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (!file || !activeTabId) return;
      const reader = new FileReader();
      reader.onload = (ev) => {
        const text = ev.target?.result;
        if (typeof text === "string") setTabSql(activeTabId, text);
      };
      reader.readAsText(file);
      e.target.value = "";
    },
    [activeTabId, setTabSql],
  );

  useEffect(() => {
    const unsubs = [
      onQueryAction("execute", handleExecute),
      onQueryAction("explain", handleExplain),
      onQueryAction("format", handleFormat),
      onQueryAction("clear", handleClear),
      onQueryAction("cancel", handleCancel),
      onQueryAction("export", () => setExportOpen(true)),
      onQueryAction("saveQuery", () => {}),
      onQueryAction("importSql", () => fileInputRef.current?.click()),
      onQueryAction("exportSql", () => {
        if (!sql.trim()) return;
        const blob = new Blob([sql], { type: "text/sql" });
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.href = url;
        a.download = "query.sql";
        a.click();
        URL.revokeObjectURL(url);
      }),
    ];
    return () => unsubs.forEach((u) => u());
  }, [handleExecute, handleExplain, handleFormat, handleClear, handleCancel, sql]);

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
          <SqlFileOperations sql={sql} onSqlLoaded={(s) => activeTabId && setTabSql(activeTabId, s)} />
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

      <WorkspaceTabBar />

      <ResizableDock>
        <div className="h-full">
          <QueryEditor
            value={sql}
            onChange={(v) => activeTabId && setTabSql(activeTabId, v)}
            path={activeTabId ? `dbpro://query/${activeTabId}.sql` : undefined}
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
      </ResizableDock>
      </div>

      <ExportDialog
        open={exportOpen}
        onClose={() => setExportOpen(false)}
        connectionId={activeConnectionId}
        sql={sql.trim()}
      />

      <input
        ref={fileInputRef}
        type="file"
        accept=".sql,text/sql"
        className="hidden"
        onChange={handleFileImport}
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
