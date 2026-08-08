/**
 * Canonical Query Application Runtime.
 *
 * This is the SINGLE source of truth for all query lifecycle behavior.
 * Both React hooks (useExecuteQuery, useExecuteQueryMulti, etc.) and
 * Action Platform handlers (query.actions.ts) delegate to this runtime.
 *
 * The runtime owns:
 *   - status management (running/success/error/cancelled)
 *   - activeExecutionId tracking
 *   - executionStartedAt tracking
 *   - stale response guard
 *   - result / multiResults management
 *   - partial multi-query failure handling
 *   - timing computation
 *   - error normalization (QUERY_CANCELLED → cancelled status)
 *   - history recording
 *   - TanStack Query cache invalidation
 *
 * Architecture:
 *
 *   UI / Keyboard / Palette / Agent / MCP
 *                  ↓
 *           Action Platform
 *                  ↓
 *        QueryApplicationRuntime
 *                ↓
 *   Workspace + Backend + History
 */

import { useQueryHistoryStore } from "@/commons/stores/query-history.store";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";

import {
  setTabActiveExecutionId,
  setTabError,
  setTabExecutionStartedAt,
  setTabExplainPlan,
  setTabMultiResults,
  setTabResult,
  setTabStatus,
  setTabTiming,
  setTabActivePanel,
  setTabSql,
} from "../controllers/query-workspace.controller";
import { createQueryService } from "../services/query.service";

import type { ExplainPlan, MultiQueryResult, QueryResult } from "../types/query.types";
import type { QueryTiming } from "@/commons/types/workspace.types";

// ─── Helpers ─────────────────────────────────────────────────

/** Check if an error is a user-initiated cancel. */
function isCancelledError(err: unknown): boolean {
  return (err as { code?: string })?.code === "QUERY_CANCELLED";
}

/**
 * Guard: returns true if a terminal callback should be IGNORED because a newer
 * execution has already started on the same tab (stale response race).
 */
function isStaleResponse(tabId: string, executionId: string): boolean {
  const tab = useWorkspaceStore.getState().tabs.find((t) => t.id === tabId);
  return tab?.kind === "query" && tab.data.activeExecutionId !== executionId;
}

/** Compute timing from execution start and server duration. */
function computeTiming(tabId: string, serverMs: number): QueryTiming {
  const now = Date.now();
  const tab = useWorkspaceStore.getState().tabs.find((t) => t.id === tabId);
  const startedAt = tab?.kind === "query" ? (tab.data.executionStartedAt ?? now) : now;
  const totalMs = now - startedAt;
  return {
    serverMs,
    totalMs,
    fetchMs: Math.max(0, totalMs - serverMs),
    renderMs: 0,
  };
}

/** Read tab context for history recording. */
function getTabContext(tabId: string): { database: string | null; schema: string | null } {
  const tab = useWorkspaceStore.getState().tabs.find((t) => t.id === tabId);
  if (tab?.kind === "query") {
    return {
      database: tab.data.context.database ?? null,
      schema: tab.data.context.schema ?? null,
    };
  }
  return { database: null, schema: null };
}

/** Normalize error message for display. */
function normalizeErrorMessage(err: unknown): string {
  const translated = err as { userMessage?: string; technicalMessage?: string };
  if (translated.technicalMessage) {
    return `${translated.userMessage ?? "Query execution failed"}: ${translated.technicalMessage}`;
  }
  if (translated.userMessage) return translated.userMessage;
  if (err instanceof Error) return err.message;
  return "Query execution failed";
}

/** Add a history entry and invalidate TanStack cache. */
function recordHistory(
  connectionId: string,
  sql: string,
  tabId: string,
  status: "success" | "error",
  durationMs: number,
  rowCount: number,
): void {
  const ctx = getTabContext(tabId);
  useQueryHistoryStore.getState().addEntry({
    id: crypto.randomUUID(),
    connectionId,
    sql,
    executedAt: new Date().toISOString(),
    durationMs,
    rowCount,
    status,
    database: ctx.database,
    schema: ctx.schema,
  });
}

// ─── Runtime API ─────────────────────────────────────────────

/**
 * Execute a single SQL statement with full lifecycle management.
 *
 * Called by both React hooks and Action Platform.
 * Returns the raw QueryResult on success.
 * Throws on failure (caller handles error path).
 */
export async function executeQuery(params: {
  connectionId: string;
  sql: string;
  executionId: string;
  tabId: string;
}): Promise<QueryResult> {
  const { connectionId, sql, executionId, tabId } = params;
  const ctx = getTabContext(tabId);

  // Set running state.
  setTabStatus(tabId, "running");
  setTabError(tabId, null);
  setTabTiming(tabId, null);
  setTabExecutionStartedAt(tabId, Date.now());
  setTabActiveExecutionId(tabId, executionId);

  try {
    const service = createQueryService();
    const result = await service.execute(connectionId, sql, executionId, ctx.database, ctx.schema);

    // Stale response guard — if a newer execution started, skip state mutation.
    if (isStaleResponse(tabId, executionId)) return result;

    const timing = computeTiming(tabId, result.durationMs);
    setTabStatus(tabId, "success");
    setTabResult(tabId, result);
    setTabTiming(tabId, timing);
    setTabExecutionStartedAt(tabId, null);
    setTabActiveExecutionId(tabId, null);

    recordHistory(connectionId, sql, tabId, "success", result.durationMs, result.rowCount);

    return result;
  } catch (err) {
    // Stale response guard.
    if (isStaleResponse(tabId, executionId)) throw err;

    setTabActiveExecutionId(tabId, null);
    setTabExecutionStartedAt(tabId, null);

    // Normalize cancelled queries.
    if (isCancelledError(err)) {
      setTabStatus(tabId, "cancelled");
      setTabError(tabId, null);
      throw err;
    }

    setTabStatus(tabId, "error");
    const display = normalizeErrorMessage(err);
    setTabError(tabId, display);

    recordHistory(connectionId, sql, tabId, "error", 0, 0);

    throw err;
  }
}

/**
 * Execute multiple SQL statements with full lifecycle management.
 *
 * Handles partial failure: preserves successful results + reports error.
 */
export async function executeQueryMulti(params: {
  connectionId: string;
  sql: string;
  executionId: string;
  tabId: string;
}): Promise<MultiQueryResult> {
  const { connectionId, sql, executionId, tabId } = params;
  const ctx = getTabContext(tabId);

  // Set running state.
  setTabStatus(tabId, "running");
  setTabError(tabId, null);
  setTabTiming(tabId, null);
  setTabExecutionStartedAt(tabId, Date.now());
  setTabActiveExecutionId(tabId, executionId);

  try {
    const service = createQueryService();
    const data = await service.executeMulti(connectionId, sql, executionId, ctx.database, ctx.schema);

    // Stale response guard.
    if (isStaleResponse(tabId, executionId)) return data;

    const timing = computeTiming(tabId, data.totalDurationMs);

    // Check for partial failure.
    const hasPartialError = data.error !== null && data.error !== undefined;

    // Always preserve successful results (even on partial failure).
    setTabMultiResults(tabId, data.results);
    if (data.results.length > 0) {
      setTabResult(tabId, data.results[0]);
    }
    setTabTiming(tabId, timing);
    setTabExecutionStartedAt(tabId, null);
    setTabActiveExecutionId(tabId, null);

    if (hasPartialError) {
      const [stmtIdx, errorMsg] = data.error!;
      const completedCount = data.results.length;
      const message = completedCount > 0
        ? `${completedCount} result(s) completed · Statement ${stmtIdx + 1} failed: ${errorMsg}`
        : `Statement ${stmtIdx + 1} failed: ${errorMsg}`;
      setTabStatus(tabId, "error");
      setTabError(tabId, message);
    } else {
      setTabStatus(tabId, "success");
      setTabError(tabId, null);
    }

    recordHistory(
      connectionId,
      sql,
      tabId,
      hasPartialError ? "error" : "success",
      data.totalDurationMs,
      data.results.reduce((sum, r) => sum + r.rowCount, 0),
    );

    return data;
  } catch (err) {
    // Stale response guard.
    if (isStaleResponse(tabId, executionId)) throw err;

    setTabActiveExecutionId(tabId, null);
    setTabExecutionStartedAt(tabId, null);

    if (isCancelledError(err)) {
      setTabStatus(tabId, "cancelled");
      setTabError(tabId, null);
      throw err;
    }

    setTabStatus(tabId, "error");
    const display = normalizeErrorMessage(err);
    setTabError(tabId, display);

    recordHistory(connectionId, sql, tabId, "error", 0, 0);

    throw err;
  }
}

/**
 * Cancel a running query execution.
 *
 * Normalizes the cancel path — both UI and Action Platform use this.
 */
export async function cancelQuery(params: {
  tabId: string;
  executionId: string;
}): Promise<void> {
  const { tabId, executionId } = params;

  const service = createQueryService();
  await service.cancel(executionId);

  setTabStatus(tabId, "cancelled");
  setTabError(tabId, null);
  setTabActiveExecutionId(tabId, null);
  setTabExecutionStartedAt(tabId, null);
}

/**
 * Explain a query and populate the explain plan.
 */
export async function explainQuery(params: {
  connectionId: string;
  sql: string;
  tabId: string;
}): Promise<ExplainPlan> {
  const { connectionId, sql, tabId } = params;

  const service = createQueryService();
  const plan = await service.explain(connectionId, sql);

  setTabExplainPlan(tabId, plan);
  setTabActivePanel(tabId, "explain");

  return plan;
}

/**
 * Format SQL and update the target tab.
 */
export async function formatSql(params: {
  tabId: string;
  sql: string;
}): Promise<string> {
  const { tabId, sql } = params;

  const { format } = await import("sql-formatter");
  const formatted = format(sql);

  setTabSql(tabId, formatted);

  return formatted;
}
