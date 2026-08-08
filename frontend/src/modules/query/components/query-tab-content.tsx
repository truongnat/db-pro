import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { format as formatSql } from "sql-formatter";

import { ResizableDock } from "@/commons/components/resizable-dock";
import { onQueryAction, dispatchQueryAction } from "@/commons/commands/query-dispatch";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { useTranslation } from "@/commons/locales/useTranslation";
import { useSchemaCatalogStore } from "../stores/schema-catalog.store";
import { Button } from "@/components/ui/button";
import { ExportDialog } from "@/modules/export/components/export-dialog";

import { ExplainPlanView } from "./explain-plan";
import { LocalHistoryPanel } from "./local-history-panel";
import { QueryCommandBar } from "./query-command-bar";
import { QueryEditor } from "./query-editor";
import { QueryHistoryPanel } from "./query-history-panel";
import { QueryStatusBar } from "./query-status-bar";
import { ResultGrid } from "./result-grid";
import { ResultTabs } from "./result-tabs";
import { SnippetPanel } from "./snippet-panel";
import {
  useCancelQuery,
  useExecuteQueryMulti,
  useExplainPlan,
  useQueryHistory,
} from "../queries/query.queries";
import { pushLocalHistory } from "../services/local-history";
import { getDialectForConnection } from "../sql/dialect";
import { createQueryTab } from "@/commons/factories/tab-factories";
import {
  setTabSql,
  setTabSort,
  setTabActivePanel,
} from "../controllers/query-workspace.controller";
import type { Row } from "../types/query.types";

interface QueryTabContentProps {
  tabId: string;
}

export function QueryTabContent({ tabId }: QueryTabContentProps) {
  const { t } = useTranslation();

  const tab = useWorkspaceStore((s) => {
    const found = s.tabs.find((t) => t.id === tabId);
    return found?.kind === "query" ? found : undefined;
  });

  const tabConnectionId = tab?.connectionId ?? null;
  const tabData = tab?.data;
  const sql = tabData?.sql ?? "";
  const status = tabData?.status ?? "idle";
  const error = tabData?.error ?? null;
  const result = tabData?.result ?? null;
  const explainPlan = tabData?.explainPlan ?? null;
  const sort = tabData?.sort ?? { column: null, direction: null };
  const timing = tabData?.timing ?? null;
  const executionStartedAt = tabData?.executionStartedAt ?? null;

  const panelTab = tabData?.activePanel ?? "results";
  const [historySearch, setHistorySearch] = useState("");
  const [exportOpen, setExportOpen] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const executeMultiMutation = useExecuteQueryMulti();
  const explainMutation = useExplainPlan();
  const cancelMutation = useCancelQuery();
  const historyQuery = useQueryHistory(tabConnectionId ?? "");

  /** Execute a SQL fragment resolved by the editor (Ctrl+Enter / F5). */
  const handleExecuteFragment = useCallback(
    (fragmentSql: string) => {
      const targetSql = fragmentSql.trim();
      if (tabConnectionId && tabId && targetSql && status !== "running") {
        const executionId = crypto.randomUUID();
        pushLocalHistory(targetSql);
        executeMultiMutation.mutate({ connectionId: tabConnectionId, sql: targetSql, executionId, tabId });
      }
    },
    [tabConnectionId, tabId, status, executeMultiMutation],
  );

  /** Execute the entire editor content (Ctrl+Shift+Enter / toolbar Run). */
  const handleExecuteAll = useCallback(() => {
    const targetSql = sql.trim();
    if (tabConnectionId && tabId && targetSql && status !== "running") {
      const executionId = crypto.randomUUID();
      pushLocalHistory(targetSql);
      executeMultiMutation.mutate({ connectionId: tabConnectionId, sql: targetSql, executionId, tabId });
    }
  }, [tabConnectionId, tabId, sql, status, executeMultiMutation]);

  const handleCancel = useCallback(() => {
    if (tabId && tabData?.activeExecutionId) {
      cancelMutation.mutate({ tabId, executionId: tabData.activeExecutionId });
    }
  }, [tabId, tabData?.activeExecutionId, cancelMutation]);

  const handleExplain = useCallback(() => {
    if (tabConnectionId && tabId && sql.trim()) {
      const currentTabId = tabId;
      explainMutation.mutate(
        { connectionId: tabConnectionId, sql: sql.trim(), tabId: currentTabId },
        {
          onSuccess: () => {
            if (useWorkspaceStore.getState().activeTabId === currentTabId) {
              setTabActivePanel(currentTabId, "explain");
            }
          },
        },
      );
    }
  }, [tabConnectionId, tabId, sql, explainMutation]);

  const handleClear = useCallback(() => {
    setTabSql(tabId, "");
  }, [tabId]);

  const handleFormat = useCallback(() => {
    if (!sql.trim()) return;
    try {
      const formatted = formatSql(sql, {
        language: getDialectForConnection(tabConnectionId).formatterLanguage,
      });
      setTabSql(tabId, formatted);
    } catch {
      // leave SQL unchanged on format error
    }
  }, [tabId, sql, tabConnectionId]);

  const handleSort = useCallback(
    (column: string) => {
      if (sort.column === column) {
        if (sort.direction === "asc") {
          setTabSort(tabId, { column, direction: "desc" });
        } else if (sort.direction === "desc") {
          setTabSort(tabId, { column: null, direction: null });
        } else {
          setTabSort(tabId, { column, direction: "asc" });
        }
      } else {
        setTabSort(tabId, { column, direction: "asc" });
      }
    },
    [tabId, sort],
  );

  const handleSelectHistoryEntry = useCallback(
    (entrySql: string) => {
      setTabSql(tabId, entrySql);
      setTabActivePanel(tabId, "results");
    },
    [tabId],
  );

  const handleFileImport = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (!file) return;
      const reader = new FileReader();
      reader.onload = (ev) => {
        const text = ev.target?.result;
        if (typeof text === "string") setTabSql(tabId, text);
      };
      reader.readAsText(file);
      e.target.value = "";
    },
    [tabId],
  );

  useEffect(() => {
    const unsubs = [
      onQueryAction("execute", handleExecuteAll),
      onQueryAction("explain", handleExplain),
      onQueryAction("format", handleFormat),
      onQueryAction("clear", handleClear),
      onQueryAction("cancel", handleCancel),
      onQueryAction("export", () => setExportOpen(true)),
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
  }, [handleExecuteAll, handleExplain, handleFormat, handleClear, handleCancel, sql]);

  useEffect(() => {
    if (tabConnectionId) {
      useSchemaCatalogStore.getState().ensureLoaded(tabConnectionId);
    }
  }, [tabConnectionId]);

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

  if (!tabConnectionId) {
    return (
      <div className="flex items-center justify-center py-12">
        <p className="text-[13px] text-[var(--app-text-muted)]">{t("query.connectFirst")}</p>
      </div>
    );
  }

  const primaryTabs = [
    { id: "results" as const, label: t("query.results") },
    { id: "explain" as const, label: t("query.explain") },
  ];
  const secondaryTabs = [
    { id: "history" as const, label: t("query.history") },
    { id: "local-history" as const, label: t("query.localHistory") },
    { id: "snippets" as const, label: t("query.snippets") },
  ];

  const renderTabButton = (tab: { id: typeof panelTab; label: string }) => (
    <button
      key={tab.id}
      type="button"
      className={`relative h-full px-3 text-[13px] transition-colors ${
        panelTab === tab.id
          ? "text-foreground"
          : "text-[var(--app-text-muted)] hover:text-foreground"
      }`}
      onClick={() => setTabActivePanel(tabId, tab.id)}
    >
      {tab.label}
      {panelTab === tab.id && (
        <span className="absolute inset-x-3 bottom-0 h-[2px] bg-primary" />
      )}
    </button>
  );

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      <QueryCommandBar
        tabId={tabId}
        connectionId={tabConnectionId}
        context={tabData?.context ?? { database: null, schema: null }}
        onExecuteCurrent={() => dispatchQueryAction("executeCurrent")}
        onExecuteAll={handleExecuteAll}
        onCancel={handleCancel}
        onExplain={handleExplain}
        onClear={handleClear}
        onExport={() => setExportOpen(true)}
        onFormat={handleFormat}
        onSaveQuery={() => dispatchQueryAction("saveQuery")}
        onExportSql={() => dispatchQueryAction("exportSql")}
        onImportSql={() => dispatchQueryAction("importSql")}
        isExecuting={status === "running"}
        isExplaining={explainMutation.isPending}
        hasConnection={!!tabConnectionId}
        hasSql={!!sql.trim()}
      />

      <ResizableDock>
        <div className="h-full">
          <QueryEditor
            value={sql}
            onChange={(v) => setTabSql(tabId, v)}
            path={`dbpro://query/${tabId}.sql`}
            onExecute={handleExecuteFragment}
            onCancel={handleCancel}
          />
        </div>

        <div className="flex min-h-0 flex-1 flex-col">
          <div className="flex h-[32px] items-center border-b border-[var(--app-border-subtle)] bg-[var(--app-surface-1)]">
            {primaryTabs.map(renderTabButton)}
            <span className="mx-1 h-4 w-px bg-[var(--app-border-subtle)]" aria-hidden />
            {secondaryTabs.map(renderTabButton)}
          </div>

          <div className="min-h-0 flex-1">
            {status === "error" && error && panelTab === "results" && (
              <div className="flex flex-col items-start justify-center px-6 py-6">
                <div className="mb-2 flex items-center gap-2">
                  <div className="grid h-6 w-6 place-items-center rounded bg-destructive/15">
                    <span className="text-xs font-bold text-destructive">!</span>
                  </div>
                  <p className="text-[13px] font-medium text-foreground">{t("query.queryError")}</p>
                </div>
                <p className="mb-4 max-w-lg text-[13px] leading-relaxed text-[var(--app-text-muted)]">{error}</p>
                <Button variant="outline" size="sm" className="h-7 rounded-[5px] text-[13px]" onClick={handleExecuteAll}>
                  {t("common.actions.retry")}
                </Button>
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
                <p className="text-[13px] text-[var(--app-text-muted)]">{t("query.enterSql")}</p>
              </div>
            )}

            {panelTab === "explain" && explainPlan && (
              <ExplainPlanView plan={explainPlan} />
            )}

            {panelTab === "explain" && !explainPlan && (
              <div className="flex items-center justify-center py-12">
                <p className="text-[13px] text-[var(--app-text-muted)]">{t("query.noResults")}</p>
              </div>
            )}

            {panelTab === "history" && (
              <QueryHistoryPanel
                entries={historyQuery.data ?? []}
                search={historySearch}
                onSearchChange={setHistorySearch}
                onSelectEntry={handleSelectHistoryEntry}
                onOpenInNewTab={(entry) => {
                  if (!tabConnectionId) return;
                  // Use historical database/schema from the entry, fallback to current tab context.
                  const context = {
                    database: entry.database ?? tabData?.context?.database ?? null,
                    schema: entry.schema ?? tabData?.context?.schema ?? null,
                  };
                  const newTab = createQueryTab(tabConnectionId, {
                    sql: entry.sql,
                    context,
                  });
                  useWorkspaceStore.getState().openTab(newTab);
                }}
                isLoading={historyQuery.isLoading}
              />
            )}

            {panelTab === "local-history" && (
              <LocalHistoryPanel onSelectEntry={handleSelectHistoryEntry} />
            )}

            {panelTab === "snippets" && (
              <SnippetPanel
                onInsertSnippet={(snippetSql) => {
                  setTabSql(tabId, sql + (sql.endsWith("\n") || !sql ? "" : "\n") + snippetSql);
                }}
              />
            )}
          </div>
        </div>
      </ResizableDock>

      <ExportDialog
        open={exportOpen}
        onClose={() => setExportOpen(false)}
        connectionId={tabConnectionId}
        sql={sql.trim()}
        columns={result?.columns ?? []}
        rows={sortedRows as Row[]}
      />

      <QueryStatusBar
        tabId={tabId}
        status={status}
        executionStartedAt={executionStartedAt}
        rowCount={result?.rowCount ?? 0}
        timing={timing}
        onCancel={handleCancel}
      />

      <input
        ref={fileInputRef}
        type="file"
        accept=".sql,text/sql"
        className="hidden"
        onChange={handleFileImport}
      />
    </div>
  );
}

function getSortValue(cell: { type: string; value?: unknown } | undefined): string | number {
  if (!cell || cell.type === "null") return "";
  if (cell.type === "int64" || cell.type === "float64") return cell.value as number;
  return String(cell.value ?? "");
}
