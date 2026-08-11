import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { ResizableDock } from "@/commons/components/resizable-dock";
import { onQueryAction, dispatchQueryAction } from "@/commons/commands/query-dispatch";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { useTranslation } from "@/commons/locales/useTranslation";
import { executeAction } from "@/commons/actions/bus";
import { useActionConfirmationStore } from "@/commons/stores/action-confirmation.store";
import { useSchemaCatalogStore } from "../stores/schema-catalog.store";
import { Button } from "@/components/ui/button";
import { useSnackbar } from "@/app/providers/snackbar.provider";
import { useConfirmDialog } from "@/app/providers/confirm-dialog.provider";
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
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { ChevronDown, History, Clock, BookOpen, HelpCircle, Database } from "lucide-react";
import { useQueryHistory } from "../queries/query.queries";
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
  const snackbar = useSnackbar();
  const { confirm } = useConfirmDialog();

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

  // Action Platform execution — all meaningful query intents go through here.
  // Confirmation is routed to the global ActionConfirmationHost in AppShell.
  const [isExplaining, setIsExplaining] = useState(false);
  const [explainError, setExplainError] = useState<string | null>(null);
  const historyQuery = useQueryHistory(tabConnectionId ?? "");

  /** Execute current statement via Action Platform (Ctrl+Enter). */
  const handleExecuteFragment = useCallback(
    async (_fragmentSql: string, source: "keyboard" | "ui" = "ui") => {
      if (tabConnectionId && tabId && status !== "running") {
        const result = await executeAction("query.execute.current", { tabId }, { source });
        if (result.status === "confirmation_required" && result.confirmation) {
          useActionConfirmationStore.getState().setPending(result.confirmation);
        }
      }
    },
    [tabConnectionId, tabId, status],
  );

  /** Execute all statements via Action Platform (Ctrl+Shift+Enter / F5 / toolbar). */
  const handleExecuteAll = useCallback(
    async (source: "keyboard" | "ui" = "ui") => {
      if (tabConnectionId && tabId && sql.trim() && status !== "running") {
        const result = await executeAction("query.execute.all", { tabId }, { source });
        if (result.status === "confirmation_required" && result.confirmation) {
          useActionConfirmationStore.getState().setPending(result.confirmation);
        }
      }
    },
    [tabConnectionId, tabId, sql, status],
  );

  /** Cancel via Action Platform. */
  const handleCancel = useCallback(
    (source: "keyboard" | "ui" = "ui") => {
      if (tabId) {
        void executeAction("query.cancel", { tabId }, { source });
      }
    },
    [tabId],
  );

  /** Explain via Action Platform — no direct hook bypass. */
  const handleExplain = useCallback(async () => {
    if (tabConnectionId && tabId && sql.trim()) {
      setIsExplaining(true);
      setExplainError(null);
      const result = await executeAction("query.explain", { tabId }, { source: "ui" });
      if (result.status === "error") {
        setExplainError(result.error?.message ?? "Explain failed");
      }
      setIsExplaining(false);
    }
  }, [tabConnectionId, tabId, sql]);

  const handleClear = useCallback(() => {
    setTabSql(tabId, "");
  }, [tabId]);

  /** Format via Action Platform. */
  const handleFormat = useCallback(() => {
    if (sql.trim() && tabId) {
      void executeAction("query.format", { tabId }, { source: "ui" });
    }
  }, [tabId, sql]);

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

  const tabDirty = tab?.dirty ?? false;

  const handleSelectHistoryEntry = useCallback(
    async (entrySql: string) => {
      if (tabDirty && sql.trim() && sql !== entrySql) {
        const ok = await confirm({
          title: t("query.dirtyReplaceConfirm"),
          message: t("query.dirtyReplaceConfirm"),
        });
        if (!ok) return;
      }
      setTabSql(tabId, entrySql);
      setTabActivePanel(tabId, "results");
    },
    [tabId, tabDirty, sql, t, confirm],
  );

  const handleFileImport = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (!file) return;
      const reader = new FileReader();
      reader.onload = async (ev) => {
        const text = ev.target?.result;
        if (typeof text === "string") {
          if (tabDirty && sql.trim()) {
            const ok = await confirm({
              title: t("query.dirtyReplaceConfirm"),
              message: t("query.dirtyReplaceConfirm"),
            });
            if (!ok) return;
          }
          setTabSql(tabId, text);
        }
      };
      reader.readAsText(file);
      e.target.value = "";
    },
    [tabId, tabDirty, sql, t, confirm],
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
      onQueryAction("saveQuery", () => {
        snackbar.info(t("query.saveComingSoon"));
      }),
    ];
    return () => unsubs.forEach((u) => u());
  }, [handleExecuteAll, handleExplain, handleFormat, handleClear, handleCancel, sql, snackbar, t]);

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
      if (aVal === bVal) return 0;
      if (aVal === "") return 1;
      if (bVal === "") return -1;
      if (typeof aVal === "bigint" && typeof bVal === "bigint") return aVal < bVal ? -1 : 1;
      if (typeof aVal === "number" && typeof bVal === "number") return aVal - bVal;
      return String(aVal).localeCompare(String(bVal));
    });
    return sort.direction === "desc" ? sorted.reverse() : sorted;
  }, [result, sort]);

  if (!tabConnectionId) {
    return (
      <div className="flex items-center justify-center py-12">
        <p className="text-[13px] text-[var(--text-secondary)]">{t("query.connectFirst")}</p>
      </div>
    );
  }

  const primaryTabs = [
    { id: "results" as const, label: t("query.results") },
    { id: "explain" as const, label: t("query.explain") },
    { id: "messages" as const, label: t("query.messages") },
  ];

  const renderTabButton = (tab: { id: typeof panelTab; label: string }) => (
    <button
      key={tab.id}
      type="button"
      className={`relative h-full px-3.5 text-[13px] transition-colors ${
        panelTab === tab.id
          ? "font-medium text-foreground"
          : "text-[var(--text-secondary)] hover:text-foreground"
      }`}
      onClick={() => setTabActivePanel(tabId, tab.id)}
    >
      {tab.label}
      {panelTab === tab.id && <span className="absolute inset-x-3 bottom-0 h-[2px] bg-primary" />}
    </button>
  );

  const secondaryTabLabels: Record<
    string,
    { label: string; icon: React.ComponentType<{ className?: string }> }
  > = {
    history: { label: t("query.history"), icon: History },
    "local-history": { label: t("query.localHistory"), icon: Clock },
    snippets: { label: t("query.snippets"), icon: BookOpen },
  };

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
        onExportSql={() => dispatchQueryAction("exportSql")}
        onImportSql={() => dispatchQueryAction("importSql")}
        isExecuting={status === "running"}
        isExplaining={isExplaining}
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
            onExecuteAll={handleExecuteAll}
            onCancel={handleCancel}
          />
        </div>

        <div className="flex min-h-0 flex-1 flex-col">
          <div className="flex h-[34px] items-center border-b border-[var(--border-subtle)] bg-[var(--surface-nav)]">
            {primaryTabs.map(renderTabButton)}
            <div className="flex-1" />
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <button
                  type="button"
                  className={`flex h-full items-center gap-1 px-3 text-[13px] transition-colors ${
                    ["history", "local-history", "snippets"].includes(panelTab)
                      ? "font-medium text-foreground"
                      : "text-[var(--text-secondary)] hover:text-foreground"
                  }`}
                >
                  {secondaryTabLabels[panelTab]?.label ?? "More"}
                  <ChevronDown className="h-3 w-3 opacity-60" />
                </button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end" className="min-w-[180px]">
                {Object.entries(secondaryTabLabels).map(([id, { label, icon: Icon }]) => (
                  <DropdownMenuItem
                    key={id}
                    className="h-[30px]"
                    onClick={() => setTabActivePanel(tabId, id as typeof panelTab)}
                  >
                    <Icon className="mr-2 h-3.5 w-3.5" />
                    {label}
                  </DropdownMenuItem>
                ))}
              </DropdownMenuContent>
            </DropdownMenu>
          </div>

          <div className="min-h-0 flex-1">
            {status === "error" && error && panelTab === "results" && (
              <div className="flex flex-col items-start justify-center px-6 py-6">
                <div className="mb-2 flex items-center gap-2">
                  <div className="grid h-7 w-7 place-items-center rounded-md bg-destructive/15">
                    <span className="text-[13px] font-bold text-destructive">!</span>
                  </div>
                  <p className="text-[13px] font-medium text-foreground">{t("query.queryError")}</p>
                </div>
                <p className="mb-4 max-w-lg text-[13px] leading-relaxed text-[var(--text-secondary)]">
                  {error}
                </p>
                <div className="flex items-center gap-2">
                  <Button
                    variant="outline"
                    size="sm"
                    className="h-7 rounded-[5px] text-[13px]"
                    onClick={() => {
                      void handleExecuteFragment("", "ui");
                    }}
                  >
                    {t("query.runCurrent")}
                  </Button>
                </div>
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
              <div className="flex flex-col items-center justify-center py-12">
                <div className="mb-3 grid h-9 w-9 place-items-center rounded-lg bg-[var(--surface-panel)]">
                  <Database className="h-4 w-4 text-[var(--text-secondary)]" />
                </div>
                <p className="mb-1 text-[13px] font-medium text-foreground">No query results yet</p>
                <p className="text-[12px] text-[var(--text-secondary)]">
                  Run the current statement to see results here.
                </p>
              </div>
            )}

            {panelTab === "explain" && explainPlan && <ExplainPlanView plan={explainPlan} />}

            {panelTab === "explain" && explainError && (
              <div className="flex flex-col items-start px-6 py-6">
                <div className="mb-2 flex items-center gap-2">
                  <div className="grid h-7 w-7 place-items-center rounded-md bg-destructive/15">
                    <span className="text-[13px] font-bold text-destructive">!</span>
                  </div>
                  <p className="text-[13px] font-medium text-foreground">Explain failed</p>
                </div>
                <p className="mb-4 max-w-lg text-[13px] leading-relaxed text-[var(--text-secondary)]">
                  {explainError}
                </p>
                <Button
                  variant="outline"
                  size="sm"
                  className="h-7 rounded-[5px] text-[13px]"
                  onClick={handleExplain}
                  disabled={!tabConnectionId || !sql.trim() || isExplaining}
                >
                  {t("common.actions.retry")}
                </Button>
              </div>
            )}

            {panelTab === "explain" && !explainPlan && !explainError && (
              <div className="flex flex-col items-center justify-center py-12">
                <div className="mb-3 grid h-9 w-9 place-items-center rounded-lg bg-[var(--surface-panel)]">
                  <HelpCircle className="h-4 w-4 text-[var(--text-secondary)]" />
                </div>
                <p className="mb-1 text-[13px] font-medium text-foreground">
                  No execution plan yet
                </p>
                <p className="mb-4 max-w-xs text-center text-[12px] leading-relaxed text-[var(--text-secondary)]">
                  Run Explain to inspect how PostgreSQL plans the current statement.
                </p>
                <Button
                  variant="outline"
                  size="sm"
                  className="h-7 rounded-[5px] text-[13px]"
                  onClick={handleExplain}
                  disabled={!tabConnectionId || !sql.trim() || isExplaining}
                >
                  <HelpCircle className="mr-1.5 h-3.5 w-3.5" />
                  {t("query.explain")}
                </Button>
              </div>
            )}

            {panelTab === "messages" && (
              <div className="flex flex-col gap-2 px-4 py-4">
                {status === "success" && timing ? (
                  <>
                    <div className="flex items-center gap-2 text-[13px] text-foreground">
                      <span className="h-2 w-2 rounded-full bg-[var(--state-success)]" />
                      Query completed
                    </div>
                    <div className="flex flex-col gap-1 text-[12px] text-[var(--text-secondary)]">
                      <span>{t("query.rowsAffected", { count: result?.rowCount ?? 0 })}</span>
                      <span>{t("query.duration", { duration: timing.totalMs })}</span>
                      {timing.serverMs > 0 && <span>Server: {timing.serverMs}ms</span>}
                    </div>
                  </>
                ) : status === "error" ? (
                  <div className="flex items-center gap-2 text-[13px] text-destructive">
                    <span className="h-2 w-2 rounded-full bg-destructive" />
                    {error ?? t("query.statusError")}
                  </div>
                ) : status === "cancelled" ? (
                  <div className="flex items-center gap-2 text-[13px] text-[var(--text-secondary)]">
                    <span className="h-2 w-2 rounded-full bg-[var(--text-tertiary)]" />
                    Execution cancelled
                  </div>
                ) : (
                  <div className="text-[13px] text-[var(--text-secondary)]">No messages yet.</div>
                )}
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

function getSortValue(
  cell: { type: string; value?: unknown } | undefined,
): string | number | bigint {
  if (!cell || cell.type === "null") return "";
  if (cell.type === "int64") {
    try {
      return BigInt(cell.value as string);
    } catch {
      return 0n;
    }
  }
  if (cell.type === "float64") return cell.value as number;
  return String(cell.value ?? "");
}
