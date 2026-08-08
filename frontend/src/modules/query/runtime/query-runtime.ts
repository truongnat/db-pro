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
 *   - history recording (from snapshot — not live tab context)
 *   - local history recording
 *   - cache invalidation signaling (for all invocation sources)
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
import { pushLocalHistory } from "../services/local-history";

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

// ─── Cache invalidation signaling ────────────────────────────

/**
 * Callbacks registered by the React layer to invalidate TanStack Query
 * caches after execution. This ensures that query history panel updates
 * after Action/MCP invocations — not just React hook invocations.
 */
const cacheInvalidationCallbacks: Array<(connectionId: string) => void> = [];

/**
 * Register a callback to be called after query execution for cache
 * invalidation. Called by the React layer during app initialization.
 */
export function registerCacheInvalidation(
  callback: (connectionId: string) => void,
): () => void {
  cacheInvalidationCallbacks.push(callback);
  return () => {
    const idx = cacheInvalidationCallbacks.indexOf(callback);
    if (idx >= 0) cacheInvalidationCallbacks.splice(idx, 1);
  };
}

function notifyCacheInvalidation(connectionId: string): void {
  for (const cb of cacheInvalidationCallbacks) {
    try { cb(connectionId); } catch { /* ignore */ }
  }
}

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

/**
 * Record a history entry from the EXECUTION SNAPSHOT — not from the live tab.
 *
 * The database/schema parameters come from the ResolvedQueryExecution
 * that was frozen at invocation time. This ensures that changing the
 * tab context while a query is running does NOT alter the historical
 * execution context.
 */
function recordHistory(
  connectionId: string,
  database: string | null,
  schema: string | null,
  sql: string,
  tabId: string,
  status: "success" | "error",
  durationMs: number,
  rowCount: number,
): void {
  useQueryHistoryStore.getState().addEntry({
    id: crypto.randomUUID(),
    connectionId,
    sql,
    executedAt: new Date().toISOString(),
    durationMs,
    rowCount,
    status,
    database,
    schema,
  });
}

// ─── Runtime API ─────────────────────────────────────────────

/**
 * Execute a single SQL statement with full lifecycle management.
 *
 * Called by both React hooks and Action Platform.
 * Returns the raw QueryResult on success.
 * Throws on failure (caller handles error path).
 *
 * IMPORTANT: database and schema are passed explicitly from the
 * ResolvedQueryExecution snapshot — NOT derived from the live tab.
 */
export async function executeQuery(params: {
  connectionId: string;
  database: string | null;
  schema: string | null;
  sql: string;
  executionId: string;
  tabId: string;
}): Promise<QueryResult> {
  const { connectionId, database, schema, sql, executionId, tabId } = params;

  // Set running state.
  setTabStatus(tabId, "running");
  setTabError(tabId, null);
  setTabTiming(tabId, null);
  setTabExecutionStartedAt(tabId, Date.now());
  setTabActiveExecutionId(tabId, executionId);

  // Record local history (all invocation sources).
  pushLocalHistory(sql);

  try {
    const service = createQueryService();
    const result = await service.execute(connectionId, sql, executionId, database, schema);

    // Stale response guard — if a newer execution started, skip state mutation.
    if (isStaleResponse(tabId, executionId)) return result;

    const timing = computeTiming(tabId, result.durationMs);
    setTabStatus(tabId, "success");
    setTabResult(tabId, result);
    setTabTiming(tabId, timing);
    setTabExecutionStartedAt(tabId, null);
    setTabActiveExecutionId(tabId, null);

    // Record history from snapshot (not live tab).
    recordHistory(connectionId, database, schema, sql, tabId, "success", result.durationMs, result.rowCount);
    notifyCacheInvalidation(connectionId);

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

    recordHistory(connectionId, database, schema, sql, tabId, "error", 0, 0);
    notifyCacheInvalidation(connectionId);

    throw err;
  }
}

/**
 * Execute multiple SQL statements with full lifecycle management.
 *
 * Handles partial failure: preserves successful results + reports error.
 *
 * IMPORTANT: database and schema come from the frozen snapshot.
 */
export async function executeQueryMulti(params: {
  connectionId: string;
  database: string | null;
  schema: string | null;
  sql: string;
  executionId: string;
  tabId: string;
}): Promise<MultiQueryResult> {
  const { connectionId, database, schema, sql, executionId, tabId } = params;

  // Set running state.
  setTabStatus(tabId, "running");
  setTabError(tabId, null);
  setTabTiming(tabId, null);
  setTabExecutionStartedAt(tabId, Date.now());
  setTabActiveExecutionId(tabId, executionId);

  // Record local history (all invocation sources).
  pushLocalHistory(sql);

  try {
    const service = createQueryService();
    const data = await service.executeMulti(connectionId, sql, executionId, database, schema);

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
      database,
      schema,
      sql,
      tabId,
      hasPartialError ? "error" : "success",
      data.totalDurationMs,
      data.results.reduce((sum, r) => sum + r.rowCount, 0),
    );
    notifyCacheInvalidation(connectionId);

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

    recordHistory(connectionId, database, schema, sql, tabId, "error", 0, 0);
    notifyCacheInvalidation(connectionId);

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
 * Format SQL using the dialect-aware formatter language.
 *
 * Uses the target connection's SqlDialect formatterLanguage so that
 * UI, Command Palette, Agent, and future MCP all produce the same
 * formatting output.
 */
export async function formatSql(params: {
  tabId: string;
  connectionId: string;
  sql: string;
}): Promise<string> {
  const { tabId, connectionId, sql } = params;

  const { getDialectForConnection } = await import("../sql/dialect");
  const { format } = await import("sql-formatter");

  const language = getDialectForConnection(connectionId).formatterLanguage;
  const formatted = format(sql, { language });

  setTabSql(tabId, formatted);

  return formatted;
}
