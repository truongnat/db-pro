import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { format as formatSql } from "sql-formatter";

import { ResizableDock } from "@/commons/components/resizable-dock";
import { onQueryAction } from "@/commons/commands/query-dispatch";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { useTranslation } from "@/commons/locales/useTranslation";
import { useSchemaCatalogStore } from "../stores/schema-catalog.store";
import { Button } from "@/components/ui/button";
import { ExportDialog } from "@/modules/export/components/export-dialog";

import { ExplainPlanView } from "./explain-plan";
import { LocalHistoryPanel } from "./local-history-panel";
import { QueryEditor } from "./query-editor";
import { QueryHistoryPanel } from "./query-history-panel";
import { QueryToolbar } from "./query-toolbar";
import { ResultGrid } from "./result-grid";
import { ResultTabs } from "./result-tabs";
import { TransactionBar } from "./transaction-bar";
import {
  useCancelQuery,
  useExecuteQueryMulti,
  useExplainPlan,
  useQueryHistory,
} from "../queries/query.queries";
import { pushLocalHistory } from "../services/local-history";
import {
  setTabSql,
  setTabSort,
  setTabActivePanel,
} from "../controllers/query-workspace.controller";
import type { Row, RunConfig } from "../types/query.types";

interface QueryTabContentProps {
  tabId: string;
  onOpenRunConfig: () => void;
}

export function QueryTabContent({ tabId, onOpenRunConfig }: QueryTabContentProps) {
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

  const panelTab = tabData?.activePanel ?? "results";
  const [historySearch, setHistorySearch] = useState("");
  const [exportOpen, setExportOpen] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const executeMultiMutation = useExecuteQueryMulti();
  const explainMutation = useExplainPlan();
  const cancelMutation = useCancelQuery();
  const historyQuery = useQueryHistory(tabConnectionId ?? "");

  const handleExecute = useCallback(() => {
    if (tabConnectionId && tabId && sql.trim()) {
      pushLocalHistory(sql.trim());
      executeMultiMutation.mutate({ connectionId: tabConnectionId, sql: sql.trim(), tabId });
    }
  }, [tabConnectionId, tabId, sql, executeMultiMutation]);

  const handleCancel = useCallback(() => {
    if (tabConnectionId && tabId) {
      cancelMutation.mutate({ connectionId: tabConnectionId, tabId });
    }
  }, [tabConnectionId, tabId, cancelMutation]);

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
      const formatted = formatSql(sql, { language: "postgresql" });
      setTabSql(tabId, formatted);
    } catch {
      // leave SQL unchanged on format error
    }
  }, [tabId, sql]);

  const handleInsertTemplate = useCallback(
    (templateSql: string) => {
      setTabSql(tabId, sql.trim() ? `${sql}\n${templateSql}` : templateSql);
    },
    [tabId, sql],
  );

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
      onQueryAction("execute", handleExecute),
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
  }, [handleExecute, handleExplain, handleFormat, handleClear, handleCancel, sql]);

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
        <p className="text-muted-foreground">{t("query.connectFirst")}</p>
      </div>
    );
  }

  const panelTabs = [
    { id: "results" as const, label: t("query.results") },
    { id: "explain" as const, label: t("query.explain") },
    { id: "history" as const, label: t("query.history") },
    { id: "local-history" as const, label: t("query.localHistory") },
  ];

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      <QueryToolbar
        onExecute={handleExecute}
        onCancel={handleCancel}
        onExplain={handleExplain}
        onClear={handleClear}
        onExport={() => setExportOpen(true)}
        onFormat={handleFormat}
        onInsertTemplate={handleInsertTemplate}
        isExecuting={status === "running"}
        isExplaining={explainMutation.isPending}
        hasConnection={!!tabConnectionId}
        hasSql={!!sql.trim()}
      />

      <TransactionBar />

      <ResizableDock>
        <div className="h-full">
          <QueryEditor
            value={sql}
            onChange={(v) => setTabSql(tabId, v)}
            path={`dbpro://query/${tabId}.sql`}
            onExecute={handleExecute}
          />
        </div>

        <div className="flex min-h-0 flex-1 flex-col">
          <div className="flex border-b border-border">
            {panelTabs.map((tab) => (
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
                onClick={() => setTabActivePanel(tabId, tab.id)}
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
                <p className="text-muted-foreground">{t("query.enterSql")}</p>
              </div>
            )}

            {panelTab === "explain" && explainPlan && (
              <ExplainPlanView plan={explainPlan} />
            )}

            {panelTab === "explain" && !explainPlan && (
              <div className="flex items-center justify-center py-12">
                <p className="text-muted-foreground">{t("query.noResults")}</p>
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

      <ExportDialog
        open={exportOpen}
        onClose={() => setExportOpen(false)}
        connectionId={tabConnectionId}
        sql={sql.trim()}
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
