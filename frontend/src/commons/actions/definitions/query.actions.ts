import { z } from "zod";

import { defineAction } from "../registry";
import { bindExternalExecutionId } from "../bus";
import { createQueryService } from "@/modules/query/services/query.service";
import { getActiveQueryTab, getQueryTabData, setTabSql, setTabStatus, setTabResult, setTabError, setTabExplainPlan, setTabActivePanel, setTabTiming, setTabExecutionStartedAt, setTabActiveExecutionId, setTabMultiResults } from "@/modules/query/controllers/query-workspace.controller";
import { resolveRunTarget } from "@/modules/query/services/statement-splitter";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { useQueryEditorContextStore } from "@/commons/stores/query-editor-context.store";

import type { ActionExecutionContext, ActionResult, ActionRisk, ResolvedQueryExecution } from "../types";
import type { ExplainPlan, QueryResult } from "@/modules/query/types/query.types";

// ─── Helpers ─────────────────────────────────────────────────

/**
 * Resolve the query tab from explicit input.tabId or fall back to active.
 * Returns full tab info including connectionId for context resolution.
 */
function resolveQueryTab(input: { tabId?: string }) {
  if (input.tabId) {
    const data = getQueryTabData(input.tabId);
    if (data) {
      const tab = useWorkspaceStore.getState().tabs.find((t) => t.id === input.tabId);
      if (tab) return { id: tab.id, data, connectionId: tab.connectionId };
    }
  }
  const active = getActiveQueryTab();
  return active ? { id: active.id, data: active.data, connectionId: active.connectionId } : undefined;
}

/**
 * Resolve the canonical executable query object.
 *
 * This is the SINGLE source of truth for the SQL that will be sent to
 * the backend. Risk classification, confirmation, execution, audit and
 * history ALL use the same resolved SQL value.
 *
 * Never resolve SQL twice in two places.
 */
function resolveExecutableQuery(
  input: { tabId?: string; cursorOffset?: number; selection?: { start: number; end: number } | null },
  ctx: ActionExecutionContext,
  mode: "current" | "selection" | "all",
): ResolvedQueryExecution | null {
  const tab = resolveQueryTab(input);
  if (!tab || !tab.connectionId) return null;

  let sql: string;

  if (mode === "selection" && (input as { sql?: string }).sql) {
    sql = (input as { sql: string }).sql;
  } else if (mode === "current") {
    // Use editor cursor context store for accurate statement resolution.
    const editorCtx = useQueryEditorContextStore.getState().getEditorContext(tab.id);
    const cursorOffset = input.cursorOffset ?? editorCtx.cursorOffset;
    const selection = input.selection !== undefined ? input.selection : editorCtx.selection;

    sql = resolveRunTarget({
      value: tab.data.sql,
      selection,
      cursorOffset,
    }) ?? tab.data.sql;
  } else {
    sql = tab.data.sql;
  }

  if (!sql.trim()) return null;

  return {
    tabId: tab.id,
    connectionId: tab.connectionId,
    database: tab.data.context.database ?? null,
    schema: tab.data.context.schema ?? null,
    sql,
    executionMode: mode,
  };
}

/**
 * Classify SQL content into a risk level.
 *
 * Handles:
 *   - Leading comments (-- and /* *\/)
 *   - WITH mutating CTE (INSERT/UPDATE/DELETE inside WITH)
 *   - EXPLAIN ANALYZE mutation
 *   - String literals (don't false-positive on keywords inside strings)
 *   - Standard DML/DDL keywords
 */
function classifySqlRisk(sql: string): ActionRisk {
  // Strip leading block and line comments.
  let stripped = sql.trim();
  while (true) {
    if (stripped.startsWith("--")) {
      const nl = stripped.indexOf("\n");
      if (nl === -1) return "read"; // only comments
      stripped = stripped.slice(nl + 1).trim();
      continue;
    }
    if (stripped.startsWith("/*")) {
      const end = stripped.indexOf("*/");
      if (end === -1) return "read"; // unclosed comment
      stripped = stripped.slice(end + 2).trim();
      continue;
    }
    break;
  }

  if (!stripped) return "read";

  const upper = stripped.toUpperCase();

  // EXPLAIN / EXPLAIN ANALYZE — classify the inner statement.
  if (/^EXPLAIN\b/.test(upper)) {
    // Strip the EXPLAIN [ANALYZE] [VERBOSE] prefix to get the inner SQL.
    const inner = stripped
      .replace(/^EXPLAIN\b/i, "")
      .replace(/^\s+ANALYZE\b/i, "")
      .replace(/^\s+VERBOSE\b/i, "")
      .trim();
    if (inner) return classifySqlRisk(inner);
    return "read";
  }

  // WITH ... mutating CTE detection.
  // Look for INSERT/UPDATE/DELETE inside WITH blocks.
  if (/^WITH\b/i.test(upper)) {
    // Check if the WITH body contains mutating keywords.
    // This is a heuristic — we look for DML keywords after the WITH ... AS.
    const body = upper.replace(/^WITH\b[\s\S]*?\bAS\b\s*\(/, "");
    if (/\b(INSERT|UPDATE|DELETE)\b/.test(body)) return "destructive";
    // If the main query after WITH is SELECT, it's read.
    return "read";
  }

  // Strip string literals to avoid false positives on keywords inside strings.
  const noStrings = upper.replace(/'[^']*'/g, "''");

  // Destructive: DROP, TRUNCATE, DELETE
  if (/^(DROP|TRUNCATE)\b/.test(noStrings)) return "destructive";
  if (/^DELETE\b/.test(noStrings)) return "destructive";

  // Write: INSERT, UPDATE, ALTER, CREATE, REPLACE
  if (/^(INSERT|UPDATE|ALTER|CREATE|REPLACE)\b/.test(noStrings)) return "write";

  // Default: read-only
  return "read";
}

function requireConnection(ctx: ActionExecutionContext): ActionResult | null {
  if (!ctx.connectionId) {
    return {
      status: "error",
      error: { code: "connection_required", message: "No active connection" },
    };
  }
  return null;
}

function requireSql(sql: string): ActionResult | null {
  if (!sql || !sql.trim()) {
    return {
      status: "error",
      error: { code: "sql_required", message: "No SQL content" },
    };
  }
  return null;
}

/**
 * Apply workspace state updates for a successful query execution.
 * This is the canonical application runtime — the same path that
 * React hooks use, now also used by the Action Platform.
 */
function applyQueryResultToWorkspace(tabId: string, result: QueryResult): void {
  setTabStatus(tabId, "success");
  setTabResult(tabId, result);
  setTabError(tabId, null);
  setTabTiming(tabId, {
    serverMs: result.durationMs,
    totalMs: result.durationMs,
    fetchMs: 0,
    renderMs: 0,
  });
  setTabActiveExecutionId(tabId, null);
}

function applyQueryErrorToWorkspace(tabId: string, message: string): void {
  setTabStatus(tabId, "error");
  setTabError(tabId, message);
  setTabActiveExecutionId(tabId, null);
}

function applyMultiResultToWorkspace(tabId: string, results: QueryResult[]): void {
  setTabStatus(tabId, "success");
  setTabMultiResults(tabId, results);
  setTabError(tabId, null);
  setTabActiveExecutionId(tabId, null);
}

// ─── query.execute.current ───────────────────────────────────

export const executeCurrentAction = defineAction<
  { tabId?: string; cursorOffset?: number; selection?: { start: number; end: number } | null },
  { rowCount: number; durationMs: number }
>({
  id: "query.execute.current",
  title: "Run current statement",
  description:
    "Execute the SQL statement at the cursor position in the active query tab.",
  category: "query",
  inputSchema: z.object({
    tabId: z.string().optional(),
    cursorOffset: z.number().int().min(0).optional(),
    selection: z.object({ start: z.number(), end: z.number() }).nullable().optional(),
  }),
  risk: "read",
  confirmation: { mode: "destructive-only" },

  resolveContext(input, ambient) {
    const tab = resolveQueryTab(input);
    if (!tab) return {};
    return {
      tabId: tab.id,
      connectionId: tab.connectionId ?? ambient.connectionId,
      database: tab.data.context.database ?? ambient.database,
      schema: tab.data.context.schema ?? ambient.schema,
    };
  },

  resolveRisk(input, ctx) {
    const resolved = resolveExecutableQuery(input, ctx, "current");
    if (!resolved) return "read";
    return classifySqlRisk(resolved.sql);
  },

  availability(ctx) {
    const tab = resolveQueryTab({ tabId: ctx.tabId });
    const hasSql = tab ? tab.data.sql.trim().length > 0 : false;
    if (!ctx.connectionId)
      return { status: "unavailable", reason: "connection_required" };
    if (!hasSql) return { status: "unavailable", reason: "sql_required" };
    return { status: "available" };
  },

  commandInput() {
    const tab = getActiveQueryTab();
    if (!tab) return undefined;
    // Read real cursor state from the editor context store.
    const editorCtx = useQueryEditorContextStore.getState().getEditorContext(tab.id);
    return {
      tabId: tab.id,
      cursorOffset: editorCtx.cursorOffset,
      selection: editorCtx.selection,
    };
  },

  async execute(input, ctx): Promise<ActionResult<{ rowCount: number; durationMs: number }>> {
    const connErr = requireConnection(ctx);
    if (connErr) return connErr as ActionResult<{ rowCount: number; durationMs: number }>;

    const resolved = resolveExecutableQuery(input, ctx, "current");
    if (!resolved) {
      return { status: "error", error: { code: "no_tab", message: "No active query tab" } } as ActionResult<{ rowCount: number; durationMs: number }>;
    }

    const sqlErr = requireSql(resolved.sql);
    if (sqlErr) return sqlErr as ActionResult<{ rowCount: number; durationMs: number }>;

    const backendExecId = crypto.randomUUID();
    const service = createQueryService();

    // Set running state.
    setTabStatus(resolved.tabId, "running");
    setTabError(resolved.tabId, null);
    setTabExecutionStartedAt(resolved.tabId, Date.now());
    setTabActiveExecutionId(resolved.tabId, backendExecId);

    try {
      const result = await service.execute(
        resolved.connectionId,
        resolved.sql,
        backendExecId,
        resolved.database,
        resolved.schema,
      );

      // Apply result to workspace state (canonical runtime).
      applyQueryResultToWorkspace(resolved.tabId, result);

      return {
        status: "success",
        data: { rowCount: result.rowCount, durationMs: result.durationMs },
        executionId: backendExecId,
      };
    } catch (err) {
      const message = err instanceof Error ? err.message : "Query execution failed";
      applyQueryErrorToWorkspace(resolved.tabId, message);
      return {
        status: "error",
        error: { code: "query_error", message },
      } as ActionResult<{ rowCount: number; durationMs: number }>;
    }
  },
});

// ─── query.execute.selection ─────────────────────────────────

export const executeSelectionAction = defineAction<
  { tabId?: string; sql: string },
  { rowCount: number; durationMs: number }
>({
  id: "query.execute.selection",
  title: "Run selected SQL",
  description: "Execute only the selected SQL text.",
  category: "query",
  inputSchema: z.object({
    tabId: z.string().optional(),
    sql: z.string().min(1),
  }),
  risk: "read",
  confirmation: { mode: "destructive-only" },

  resolveContext(input, ambient) {
    const tab = resolveQueryTab(input);
    if (!tab) return {};
    return {
      tabId: tab.id,
      connectionId: tab.connectionId ?? ambient.connectionId,
      database: tab.data.context.database ?? ambient.database,
      schema: tab.data.context.schema ?? ambient.schema,
    };
  },

  resolveRisk(input) {
    return classifySqlRisk(input.sql);
  },

  async execute(input, ctx): Promise<ActionResult<{ rowCount: number; durationMs: number }>> {
    const connErr = requireConnection(ctx);
    if (connErr) return connErr as ActionResult<{ rowCount: number; durationMs: number }>;
    const sqlErr = requireSql(input.sql);
    if (sqlErr) return sqlErr as ActionResult<{ rowCount: number; durationMs: number }>;

    const tab = resolveQueryTab(input);
    const tabId = tab?.id ?? ctx.tabId;

    const backendExecId = crypto.randomUUID();
    const service = createQueryService();

    if (tabId) {
      setTabStatus(tabId, "running");
      setTabError(tabId, null);
      setTabExecutionStartedAt(tabId, Date.now());
      setTabActiveExecutionId(tabId, backendExecId);
    }

    try {
      const result = await service.execute(
        ctx.connectionId!,
        input.sql,
        backendExecId,
        ctx.database,
        ctx.schema,
      );

      if (tabId) applyQueryResultToWorkspace(tabId, result);

      return {
        status: "success",
        data: { rowCount: result.rowCount, durationMs: result.durationMs },
        executionId: backendExecId,
      };
    } catch (err) {
      const message = err instanceof Error ? err.message : "Query execution failed";
      if (tabId) applyQueryErrorToWorkspace(tabId, message);
      return {
        status: "error",
        error: { code: "query_error", message },
      } as ActionResult<{ rowCount: number; durationMs: number }>;
    }
  },
});

// ─── query.execute.all ───────────────────────────────────────

export const executeAllAction = defineAction<
  { tabId?: string },
  { totalDurationMs: number; statementCount: number }
>({
  id: "query.execute.all",
  title: "Run all statements",
  description: "Execute all SQL statements in the active query tab.",
  category: "query",
  inputSchema: z.object({ tabId: z.string().optional() }),
  risk: "read",
  confirmation: { mode: "destructive-only" },

  resolveContext(input, ambient) {
    const tab = resolveQueryTab(input);
    if (!tab) return {};
    return {
      tabId: tab.id,
      connectionId: tab.connectionId ?? ambient.connectionId,
      database: tab.data.context.database ?? ambient.database,
      schema: tab.data.context.schema ?? ambient.schema,
    };
  },

  resolveRisk(input, _ctx) {
    const resolved = resolveExecutableQuery(input, _ctx, "all");
    if (!resolved) return "read";
    return classifySqlRisk(resolved.sql);
  },

  commandInput() {
    const tab = getActiveQueryTab();
    return tab ? { tabId: tab.id } : undefined;
  },

  async execute(input, ctx): Promise<ActionResult<{ totalDurationMs: number; statementCount: number }>> {
    const connErr = requireConnection(ctx);
    if (connErr) return connErr as ActionResult<{ totalDurationMs: number; statementCount: number }>;

    const resolved = resolveExecutableQuery(input, ctx, "all");
    if (!resolved) {
      return { status: "error", error: { code: "no_tab", message: "No active query tab" } } as ActionResult<{ totalDurationMs: number; statementCount: number }>;
    }

    const sqlErr = requireSql(resolved.sql);
    if (sqlErr) return sqlErr as ActionResult<{ totalDurationMs: number; statementCount: number }>;

    const backendExecId = crypto.randomUUID();
    const service = createQueryService();

    setTabStatus(resolved.tabId, "running");
    setTabError(resolved.tabId, null);
    setTabExecutionStartedAt(resolved.tabId, Date.now());
    setTabActiveExecutionId(resolved.tabId, backendExecId);

    try {
      const result = await service.executeMulti(
        resolved.connectionId,
        resolved.sql,
        backendExecId,
        resolved.database,
        resolved.schema,
      );

      applyMultiResultToWorkspace(resolved.tabId, result.results);

      return {
        status: "success",
        data: {
          totalDurationMs: result.totalDurationMs,
          statementCount: result.results.length,
        },
        executionId: backendExecId,
      };
    } catch (err) {
      const message = err instanceof Error ? err.message : "Query execution failed";
      applyQueryErrorToWorkspace(resolved.tabId, message);
      return {
        status: "error",
        error: { code: "query_error", message },
      } as ActionResult<{ totalDurationMs: number; statementCount: number }>;
    }
  },
});

// ─── query.cancel ────────────────────────────────────────────

export const cancelQueryAction = defineAction<
  { executionId?: string; tabId?: string },
  void
>({
  id: "query.cancel",
  title: "Cancel query",
  description: "Cancel the currently running query execution.",
  category: "query",
  inputSchema: z.object({
    executionId: z.string().optional(),
    tabId: z.string().optional(),
  }),
  risk: "read",

  resolveContext(input, ambient) {
    const tab = resolveQueryTab(input);
    if (!tab) return {};
    return { tabId: tab.id };
  },

  availability(ctx) {
    const tab = resolveQueryTab({ tabId: ctx.tabId });
    if (!tab || tab.data.status !== "running") {
      return { status: "unavailable", reason: "no_running_query" };
    }
    return { status: "available" };
  },

  async execute(input, ctx) {
    const tab = resolveQueryTab(input);
    const execId =
      input.executionId ?? tab?.data.activeExecutionId;
    if (!execId) {
      return {
        status: "error",
        error: { code: "no_execution", message: "No running execution to cancel" },
      };
    }

    const service = createQueryService();
    await service.cancel(execId);

    const tabId = tab?.id ?? ctx.tabId;
    if (tabId) {
      setTabStatus(tabId, "cancelled");
      setTabActiveExecutionId(tabId, null);
    }

    return {
      status: "success",
      executionId: execId,
    };
  },

  async cancel(execution) {
    // The cancel hook is called by the bus for any cancelExecution().
    // Use the EXTERNAL execution ID (backend query ID), not the action-level ID.
    const externalId = execution.externalExecutionId;
    if (externalId) {
      const service = createQueryService();
      await service.cancel(externalId);
    }
  },
});

// ─── query.explain ───────────────────────────────────────────

export const explainQueryAction = defineAction<
  { tabId?: string },
  { plan: ExplainPlan }
>({
  id: "query.explain",
  title: "Explain query",
  description: "Show the query execution plan for the current SQL.",
  category: "query",
  inputSchema: z.object({ tabId: z.string().optional() }),
  risk: "read",

  resolveContext(input, ambient) {
    const tab = resolveQueryTab(input);
    if (!tab) return {};
    return {
      tabId: tab.id,
      connectionId: tab.connectionId ?? ambient.connectionId,
      database: tab.data.context.database ?? ambient.database,
      schema: tab.data.context.schema ?? ambient.schema,
    };
  },

  commandInput() {
    const tab = getActiveQueryTab();
    return tab ? { tabId: tab.id } : undefined;
  },

  async execute(input, ctx): Promise<ActionResult<{ plan: ExplainPlan }>> {
    const connErr = requireConnection(ctx);
    if (connErr) return connErr as ActionResult<{ plan: ExplainPlan }>;

    const tab = resolveQueryTab(input);
    const sql = tab?.data.sql ?? "";
    const sqlErr = requireSql(sql);
    if (sqlErr) return sqlErr as ActionResult<{ plan: ExplainPlan }>;

    const service = createQueryService();
    const plan = await service.explain(ctx.connectionId!, sql);

    // Actually update workspace state — populate explain plan and activate panel.
    const tabId = tab?.id ?? ctx.tabId;
    if (tabId) {
      setTabExplainPlan(tabId, plan);
      setTabActivePanel(tabId, "explain");
    }

    return {
      status: "success",
      data: { plan },
    };
  },
});

// ─── query.format ────────────────────────────────────────────

export const formatSqlAction = defineAction<
  { tabId?: string; sql?: string },
  { formattedSql: string }
>({
  id: "query.format",
  title: "Format SQL",
  description: "Format the SQL in the active editor.",
  category: "query",
  inputSchema: z.object({
    tabId: z.string().optional(),
    sql: z.string().min(1).optional(),
  }),
  risk: "read",

  resolveContext(input, ambient) {
    const tab = resolveQueryTab(input);
    if (!tab) return {};
    return { tabId: tab.id };
  },

  commandInput() {
    const tab = getActiveQueryTab();
    return tab ? { tabId: tab.id, sql: tab.data.sql } : undefined;
  },

  async execute(input, ctx) {
    const tab = resolveQueryTab(input);
    const sql = input.sql ?? tab?.data.sql ?? "";
    if (!sql.trim()) {
      return {
        status: "error",
        error: { code: "sql_required", message: "No SQL content to format" },
      };
    }

    // Dynamic import to avoid loading sql-formatter unless needed.
    const { format } = await import("sql-formatter");
    const formatted = format(sql);

    // Actually update the editor SQL.
    const tabId = tab?.id ?? ctx.tabId;
    if (tabId) {
      setTabSql(tabId, formatted);
    }

    return {
      status: "success",
      data: { formattedSql: formatted },
    } satisfies ActionResult<{ formattedSql: string }>;
  },
});

// ─── query.clear ─────────────────────────────────────────────

export const clearSqlAction = defineAction<{ tabId?: string }, void>({
  id: "query.clear",
  title: "Clear SQL editor",
  description: "Clear all SQL content in the active query tab.",
  category: "query",
  inputSchema: z.object({ tabId: z.string().optional() }),
  risk: "read",

  resolveContext(input, ambient) {
    const tab = resolveQueryTab(input);
    if (!tab) return {};
    return { tabId: tab.id };
  },

  async execute(input, ctx) {
    const tabId = input.tabId ?? ctx.tabId;
    if (!tabId) {
      return {
        status: "error",
        error: { code: "no_tab", message: "No active query tab" },
      };
    }

    setTabSql(tabId, "");

    return {
      status: "success",
    };
  },
});

// ─── query.save ──────────────────────────────────────────────

export const saveQueryAction = defineAction<
  { name?: string; tabId?: string },
  { savedQueryId: string }
>({
  id: "query.save",
  title: "Save query",
  description: "Save the current SQL as a named query.",
  category: "query",
  inputSchema: z.object({
    name: z.string().min(1).optional(),
    tabId: z.string().optional(),
  }),
  risk: "read",

  resolveContext(input, ambient) {
    const tab = resolveQueryTab(input);
    if (!tab) return {};
    return {
      tabId: tab.id,
      connectionId: tab.connectionId ?? ambient.connectionId,
    };
  },

  commandInput() {
    const tab = getActiveQueryTab();
    return tab ? { tabId: tab.id } : undefined;
  },

  async execute(input, ctx): Promise<ActionResult<{ savedQueryId: string }>> {
    const connErr = requireConnection(ctx);
    if (connErr) return connErr as ActionResult<{ savedQueryId: string }>;

    const tab = resolveQueryTab(input);
    const sql = tab?.data.sql ?? "";
    const sqlErr = requireSql(sql);
    if (sqlErr) return sqlErr as ActionResult<{ savedQueryId: string }>;

    // If no name provided, we can't save — return error with a hint.
    // The UI should open a dialog to collect the name.
    if (!input.name) {
      return {
        status: "error",
        error: {
          code: "name_required",
          message: "Query name is required. Use the Save dialog to provide a name.",
        },
      };
    }

    const service = createQueryService();
    const saved = await service.save(ctx.connectionId!, input.name, sql);

    return {
      status: "success",
      data: { savedQueryId: saved.id },
    };
  },
});

// ─── query.get_context ───────────────────────────────────────

export const getQueryContextAction = defineAction<
  { tabId?: string },
  {
    tabId: string | null;
    connectionId: string | null;
    database: string | null;
    schema: string | null;
    status: string;
  }
>({
  id: "query.get_context",
  title: "Get query context",
  description: "Return the connection, database, schema, and execution status of the active query tab.",
  category: "query",
  inputSchema: z.object({ tabId: z.string().optional() }),
  risk: "read",

  commandInput() {
    const tab = getActiveQueryTab();
    return tab ? { tabId: tab.id } : undefined;
  },

  async execute(input, _ctx) {
    const tab = resolveQueryTab(input);
    if (!tab) {
      return {
        status: "error",
        error: { code: "no_active_tab", message: "No active query tab" },
      };
    }

    return {
      status: "success",
      data: {
        tabId: tab.id,
        connectionId: tab.connectionId,
        database: tab.data.context.database,
        schema: tab.data.context.schema,
        status: tab.data.status,
      },
    } satisfies ActionResult;
  },
});

// ─── query.get_sql ───────────────────────────────────────────

export const getQuerySqlAction = defineAction<
  { tabId?: string },
  { tabId: string; sql: string }
>({
  id: "query.get_sql",
  title: "Get SQL content",
  description: "Return the SQL content of the active query tab.",
  category: "query",
  inputSchema: z.object({ tabId: z.string().optional() }),
  risk: "read",

  commandInput() {
    const tab = getActiveQueryTab();
    return tab ? { tabId: tab.id } : undefined;
  },

  async execute(input, _ctx) {
    const tab = resolveQueryTab(input);
    if (!tab) {
      return {
        status: "error",
        error: { code: "no_active_tab", message: "No active query tab" },
      };
    }

    return {
      status: "success",
      data: { tabId: tab.id, sql: tab.data.sql },
    } satisfies ActionResult<{ tabId: string; sql: string }>;
  },
});

// ─── query.get_result ────────────────────────────────────────

export const getQueryResultAction = defineAction<
  { tabId?: string },
  {
    tabId: string;
    rowCount: number | null;
    durationMs: number | null;
    columnCount: number | null;
    hasResult: boolean;
  }
>({
  id: "query.get_result",
  title: "Get query result",
  description: "Return summary information about the current query result.",
  category: "query",
  inputSchema: z.object({ tabId: z.string().optional() }),
  risk: "read",

  commandInput() {
    const tab = getActiveQueryTab();
    return tab ? { tabId: tab.id } : undefined;
  },

  async execute(input, _ctx) {
    const tab = resolveQueryTab(input);
    if (!tab) {
      return {
        status: "error",
        error: { code: "no_active_tab", message: "No active query tab" },
      };
    }

    const result = tab.data.result;

    return {
      status: "success",
      data: {
        tabId: tab.id,
        rowCount: result?.rowCount ?? null,
        durationMs: result?.durationMs ?? null,
        columnCount: result?.columns.length ?? null,
        hasResult: result !== null,
      },
    } satisfies ActionResult;
  },
});
