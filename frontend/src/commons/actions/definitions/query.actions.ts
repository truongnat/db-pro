import { z } from "zod";

import { defineAction } from "../registry";
import { bindExternalExecutionId } from "../bus";
import { getActiveQueryTab, getQueryTabData, setTabSql, setTabStatus } from "@/modules/query/controllers/query-workspace.controller";
import { resolveRunTarget } from "@/modules/query/services/statement-splitter";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { useQueryEditorContextStore } from "@/commons/stores/query-editor-context.store";
import { classifySqlRisk, classifyScriptRisk } from "@/modules/query/sql/sql-risk-classifier";
import {
  executeQuery as runtimeExecuteQuery,
  executeQueryMulti as runtimeExecuteQueryMulti,
  cancelQuery as runtimeCancelQuery,
  explainQuery as runtimeExplainQuery,
  formatSql as runtimeFormatSql,
} from "@/modules/query/runtime/query-runtime";

import type { ActionExecutionContext, ActionResult, ActionRisk, ResolvedQueryExecution } from "../types";
import type { ExplainPlan } from "@/modules/query/types/query.types";

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
 * Called ONCE by the bus via resolvePayload(). The result is stored on
 * ctx.resolvedPayload and reused by resolveRisk and execute — never
 * resolved twice.
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
    // Use editor cursor context store for accurate statement resolutions.
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
 * Read the frozen ResolvedQueryExecution from context.
 *
 * The bus resolves the payload ONCE and stores it on ctx.resolvedPayload.
 * Both resolveRisk and execute read from this same instance.
 */
function getResolvedPayload(ctx: ActionExecutionContext): ResolvedQueryExecution | null {
  if (!ctx.resolvedPayload) return null;
  return ctx.resolvedPayload as unknown as ResolvedQueryExecution;
}

function requireConnection(ctx: ActionExecutionContext): ActionResult | null {
  if (!ctx.connectionId) {
    return { status: "error", error: { code: "connection_required", message: "No active connection" } };
  }
  return null;
}

function requireSql(sql: string): ActionResult | null {
  if (!sql || !sql.trim()) {
    return { status: "error", error: { code: "sql_required", message: "No SQL content" } };
  }
  return null;
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

  resolvePayload(input, ctx) {
    const resolved = resolveExecutableQuery(input, ctx, "current");
    return resolved ? (resolved as unknown as Record<string, unknown>) : null;
  },

  resolveRisk(_input, ctx) {
    // Read from the already-resolved payload — never resolve twice.
    const payload = getResolvedPayload(ctx);
    if (!payload) return "read";
    return classifySqlRisk(payload.sql);
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
    const editorCtx = useQueryEditorContextStore.getState().getEditorContext(tab.id);
    return {
      tabId: tab.id,
      cursorOffset: editorCtx.cursorOffset,
      selection: editorCtx.selection,
    };
  },

  async execute(input, ctx): Promise<ActionResult<{ rowCount: number; durationMs: number }>> {
    const connErr = requireConnection(ctx);
    if (connErr) return connErr as ActionResult<never>;

    // Read the frozen payload from context — not from live workspace.
    const resolved = getResolvedPayload(ctx);
    if (!resolved) {
      return { status: "error", error: { code: "no_tab", message: "No active query tab" } } as ActionResult<never>;
    }

    const backendExecId = crypto.randomUUID();

    // Bind using actionExecutionId (NOT correlationId).
    // correlationId is for tracing only.
    if (ctx.actionExecutionId) {
      bindExternalExecutionId(ctx.actionExecutionId, backendExecId);
    }

    try {
      const result = await runtimeExecuteQuery({
        connectionId: resolved.connectionId,
        database: resolved.database,
        schema: resolved.schema,
        sql: resolved.sql,
        executionId: backendExecId,
        tabId: resolved.tabId,
      });
      return {
        status: "success",
        data: { rowCount: result.rowCount, durationMs: result.durationMs },
      };
    } catch (err) {
      if ((err as { code?: string })?.code === "QUERY_CANCELLED") {
        return { status: "cancelled" } as ActionResult<never>;
      }
      const message = err instanceof Error ? err.message : "Query execution failed";
      return { status: "error", error: { code: "query_error", message } } as ActionResult<never>;
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

  resolvePayload(input, ctx) {
    const tab = resolveQueryTab(input);
    if (!tab || !tab.connectionId) return null;
    const resolved: ResolvedQueryExecution = {
      tabId: tab.id,
      connectionId: tab.connectionId,
      database: tab.data.context.database ?? null,
      schema: tab.data.context.schema ?? null,
      sql: input.sql,
      executionMode: "selection",
    };
    return resolved as unknown as Record<string, unknown>;
  },

  resolveRisk(_input, ctx) {
    const payload = getResolvedPayload(ctx);
    if (!payload) return "read";
    return classifySqlRisk(payload.sql);
  },

  async execute(input, ctx): Promise<ActionResult<{ rowCount: number; durationMs: number }>> {
    const connErr = requireConnection(ctx);
    if (connErr) return connErr as ActionResult<never>;

    const resolved = getResolvedPayload(ctx);
    if (!resolved) {
      return { status: "error", error: { code: "no_tab", message: "No active query tab" } } as ActionResult<never>;
    }

    const backendExecId = crypto.randomUUID();
    if (ctx.actionExecutionId) {
      bindExternalExecutionId(ctx.actionExecutionId, backendExecId);
    }

    try {
      const result = await runtimeExecuteQuery({
        connectionId: resolved.connectionId,
        database: resolved.database,
        schema: resolved.schema,
        sql: resolved.sql,
        executionId: backendExecId,
        tabId: resolved.tabId,
      });
      return {
        status: "success",
        data: { rowCount: result.rowCount, durationMs: result.durationMs },
      };
    } catch (err) {
      if ((err as { code?: string })?.code === "QUERY_CANCELLED") {
        return { status: "cancelled" } as ActionResult<never>;
      }
      const message = err instanceof Error ? err.message : "Query execution failed";
      return { status: "error", error: { code: "query_error", message } } as ActionResult<never>;
    }
  },
});

// ─── query.execute.all ───────────────────────────────────────

export const executeAllAction = defineAction<
  { tabId?: string },
  { totalDurationMs: number; statementCount: number; completedResults: number; failedStatement?: number }
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

  resolvePayload(input, ctx) {
    const resolved = resolveExecutableQuery(input, ctx, "all");
    return resolved ? (resolved as unknown as Record<string, unknown>) : null;
  },

  resolveRisk(_input, ctx) {
    // Read from the already-resolved payload — never resolve twice.
    const payload = getResolvedPayload(ctx);
    if (!payload) return "read";
    // Use classifyScriptRisk for multi-statement: splits, classifies each, returns max.
    // SELECT 1; DROP TABLE users → destructive (not read).
    return classifyScriptRisk(payload.sql);
  },

  commandInput() {
    const tab = getActiveQueryTab();
    return tab ? { tabId: tab.id } : undefined;
  },

  async execute(input, ctx): Promise<ActionResult<{ totalDurationMs: number; statementCount: number; completedResults: number; failedStatement?: number }>> {
    const connErr = requireConnection(ctx);
    if (connErr) return connErr as ActionResult<never>;

    const resolved = getResolvedPayload(ctx);
    if (!resolved) {
      return { status: "error", error: { code: "no_tab", message: "No active query tab" } } as ActionResult<never>;
    }

    const backendExecId = crypto.randomUUID();
    if (ctx.actionExecutionId) {
      bindExternalExecutionId(ctx.actionExecutionId, backendExecId);
    }

    try {
      const data = await runtimeExecuteQueryMulti({
        connectionId: resolved.connectionId,
        database: resolved.database,
        schema: resolved.schema,
        sql: resolved.sql,
        executionId: backendExecId,
        tabId: resolved.tabId,
      });

      // P1-4: Check for partial failure — never return success for partial failure.
      if (data.error) {
        const [stmtIdx] = data.error;
        return {
          status: "error",
          error: {
            code: "partial_execution_failure",
            message: `Statement ${stmtIdx + 1} failed. ${data.results.length} of ${stmtIdx + 1} statements completed.`,
          },
          data: {
            totalDurationMs: data.totalDurationMs,
            statementCount: data.results.length,
            completedResults: data.results.length,
            failedStatement: stmtIdx,
          },
        };
      }

      return {
        status: "success",
        data: {
          totalDurationMs: data.totalDurationMs,
          statementCount: data.results.length,
          completedResults: data.results.length,
        },
      };
    } catch (err) {
      if ((err as { code?: string })?.code === "QUERY_CANCELLED") {
        return { status: "cancelled" } as ActionResult<never>;
      }
      const message = err instanceof Error ? err.message : "Query execution failed";
      return { status: "error", error: { code: "query_error", message } } as ActionResult<never>;
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
    const execId = input.executionId ?? tab?.data.activeExecutionId;
    if (!execId) {
      return {
        status: "error",
        error: { code: "no_execution", message: "No running execution to cancel" },
      };
    }

    const tabId = tab?.id ?? ctx.tabId;
    if (!tabId) {
      return { status: "error", error: { code: "no_tab", message: "No active tab" } };
    }

    try {
      await runtimeCancelQuery({ tabId, executionId: execId });
      return { status: "success" };
    } catch (err) {
      const message = err instanceof Error ? err.message : "Cancel failed";
      return { status: "error", error: { code: "cancel_failed", message } };
    }
  },

  async cancel(execution) {
    // The cancel hook uses the EXTERNAL execution ID (backend query ID)
    // for real cancellation at the database level.
    const externalId = execution.externalExecutionId;
    if (externalId) {
      const { cancelQuery } = await import("@/modules/query/runtime/query-runtime");
      await cancelQuery({ tabId: "", executionId: externalId });
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
    if (connErr) return connErr as ActionResult<never>;

    const tab = resolveQueryTab(input);
    const sql = tab?.data.sql ?? "";
    const sqlErr = requireSql(sql);
    if (sqlErr) return sqlErr as ActionResult<never>;

    const tabId = tab?.id ?? ctx.tabId;
    if (!tabId) {
      return { status: "error", error: { code: "no_tab", message: "No active query tab" } } as ActionResult<never>;
    }

    try {
      const plan = await runtimeExplainQuery({
        connectionId: ctx.connectionId!,
        sql,
        tabId,
      });
      return { status: "success", data: { plan } };
    } catch (err) {
      const message = err instanceof Error ? err.message : "Explain failed";
      return { status: "error", error: { code: "explain_failed", message } } as ActionResult<never>;
    }
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
    return {
      tabId: tab.id,
      connectionId: tab.connectionId ?? ambient.connectionId,
    };
  },

  commandInput() {
    const tab = getActiveQueryTab();
    return tab ? { tabId: tab.id, sql: tab.data.sql } : undefined;
  },

  async execute(input, ctx): Promise<ActionResult<{ formattedSql: string }>> {
    const tab = resolveQueryTab(input);
    const sql = input.sql ?? tab?.data.sql ?? "";
    if (!sql.trim()) {
      return {
        status: "error",
        error: { code: "sql_required", message: "No SQL content to format" },
      } as ActionResult<never>;
    }

    const tabId = tab?.id ?? ctx.tabId;
    if (!tabId) {
      return { status: "error", error: { code: "no_tab", message: "No active query tab" } } as ActionResult<never>;
    }

    // P1-6: Pass connectionId for dialect-aware formatting.
    const connectionId = ctx.connectionId;
    if (!connectionId) {
      return { status: "error", error: { code: "connection_required", message: "No active connection" } } as ActionResult<never>;
    }

    try {
      const formatted = await runtimeFormatSql({ tabId, connectionId, sql });
      return { status: "success", data: { formattedSql: formatted } };
    } catch (err) {
      const message = err instanceof Error ? err.message : "Format failed";
      return { status: "error", error: { code: "format_failed", message } } as ActionResult<never>;
    }
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

    const { createQueryService } = await import("@/modules/query/services/query.service");
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
