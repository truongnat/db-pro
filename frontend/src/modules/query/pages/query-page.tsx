import { useCallback, useMemo } from "react";

import { useConnectionStore } from "@/commons/stores/connection.store";
import { useTranslation } from "@/commons/locales/useTranslation";

import { ExplainPlanView } from "../components/explain-plan";
import { QueryEditor } from "../components/query-editor";
import { QueryHistoryPanel } from "../components/query-history-panel";
import { QueryToolbar } from "../components/query-toolbar";
import { ResultGrid } from "../components/result-grid";
import { TransactionBar } from "../components/transaction-bar";
import {
  useExecuteQuery,
  useExplainPlan,
  useQueryHistory,
} from "../queries/query.queries";
import { useQueryModuleStore } from "../state/query.store";
import type { Row } from "../types/query.types";

export function QueryPage() {
  const { t } = useTranslation();
  const activeConnectionId = useConnectionStore((s) => s.activeConnectionId);

  const sql = useQueryModuleStore((s) => s.sql);
  const setSql = useQueryModuleStore((s) => s.setSql);
  const status = useQueryModuleStore((s) => s.status);
  const error = useQueryModuleStore((s) => s.error);
  const result = useQueryModuleStore((s) => s.result);
  const explainPlan = useQueryModuleStore((s) => s.explainPlan);
  const activeTab = useQueryModuleStore((s) => s.activeTab);
  const setActiveTab = useQueryModuleStore((s) => s.setActiveTab);
  const sort = useQueryModuleStore((s) => s.sort);
  const setSort = useQueryModuleStore((s) => s.setSort);
  const historySearch = useQueryModuleStore((s) => s.historySearch);
  const setHistorySearch = useQueryModuleStore((s) => s.setHistorySearch);

  const executeMutation = useExecuteQuery();
  const explainMutation = useExplainPlan();
  const historyQuery = useQueryHistory(activeConnectionId ?? "");

  const handleExecute = useCallback(() => {
    if (activeConnectionId && sql.trim()) {
      executeMutation.mutate({ connectionId: activeConnectionId, sql: sql.trim() });
    }
  }, [activeConnectionId, sql, executeMutation]);

  const handleExplain = useCallback(() => {
    if (activeConnectionId && sql.trim()) {
      explainMutation.mutate({ connectionId: activeConnectionId, sql: sql.trim() });
    }
  }, [activeConnectionId, sql, explainMutation]);

  const handleClear = useCallback(() => {
    setSql("");
  }, [setSql]);

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
      setActiveTab("results");
    },
    [setSql, setActiveTab],
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
        <p style={{ color: "var(--color-text-secondary)" }}>
          {t("query.connectFirst")}
        </p>
      </div>
    );
  }

  const tabs = [
    { id: "results" as const, label: t("query.results") },
    { id: "explain" as const, label: t("query.explain") },
    { id: "history" as const, label: t("query.history") },
  ];

  return (
    <div className="flex h-full flex-col">
      <QueryToolbar
        onExecute={handleExecute}
        onExplain={handleExplain}
        onClear={handleClear}
        isExecuting={executeMutation.isPending}
        isExplaining={explainMutation.isPending}
        hasConnection={!!activeConnectionId}
        hasSql={!!sql.trim()}
      />

      <TransactionBar />

      <div className="h-[35%] min-h-[120px] border-b">
        <QueryEditor
          value={sql}
          onChange={setSql}
          onExecute={handleExecute}
        />
      </div>

      <div className="flex min-h-0 flex-1 flex-col">
        <div
          className="flex border-b"
          style={{ borderColor: "var(--color-border)" }}
        >
          {tabs.map((tab) => (
            <button
              key={tab.id}
              className="px-4 py-2 text-sm transition-colors"
              style={{
                color:
                  activeTab === tab.id
                    ? "var(--color-text)"
                    : "var(--color-text-secondary)",
                borderBottom:
                  activeTab === tab.id
                    ? "2px solid var(--color-primary,#3b82f6)"
                    : "2px solid transparent",
              }}
              onClick={() => setActiveTab(tab.id)}
            >
              {tab.label}
            </button>
          ))}
        </div>

        <div className="min-h-0 flex-1">
          {status === "error" && error && activeTab === "results" && (
            <div
              className="m-3 rounded-[var(--radius-sm)] px-3 py-2 text-sm"
              style={{
                backgroundColor: "var(--color-error,#ef4444)",
                color: "white",
              }}
            >
              {error}
            </div>
          )}

          {activeTab === "results" && result && (
            <ResultGrid
              columns={result.columns}
              rows={sortedRows as Row[]}
              sort={sort}
              onSort={handleSort}
              durationMs={result.durationMs}
              rowCount={result.rowCount}
            />
          )}

          {activeTab === "results" && !result && status !== "error" && (
            <div className="flex items-center justify-center py-12">
              <p style={{ color: "var(--color-text-secondary)" }}>
                {t("query.enterSql")}
              </p>
            </div>
          )}

          {activeTab === "explain" && explainPlan && (
            <ExplainPlanView plan={explainPlan} />
          )}

          {activeTab === "explain" && !explainPlan && (
            <div className="flex items-center justify-center py-12">
              <p style={{ color: "var(--color-text-secondary)" }}>
                {t("query.noResults")}
              </p>
            </div>
          )}

          {activeTab === "history" && (
            <QueryHistoryPanel
              entries={historyQuery.data ?? []}
              search={historySearch}
              onSearchChange={setHistorySearch}
              onSelectEntry={handleSelectHistoryEntry}
              isLoading={historyQuery.isLoading}
            />
          )}
        </div>
      </div>
    </div>
  );
}

function getSortValue(cell: { type: string; value?: unknown } | undefined): string | number {
  if (!cell || cell.type === "null") return "";
  if (cell.type === "int64" || cell.type === "float64") return cell.value as number;
  return String(cell.value ?? "");
}
