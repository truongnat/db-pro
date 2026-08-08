import { z } from "zod";

import { defineAction } from "../registry";
import { createQueryService } from "@/modules/query/services/query.service";
import { getActiveQueryTab } from "@/modules/query/controllers/query-workspace.controller";

import type { ActionExecutionContext, ActionResult } from "../types";
import type { ExplainPlan } from "@/modules/query/types/query.types";

// ─── Helpers ─────────────────────────────────────────────────

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

// ─── query.execute.current ───────────────────────────────────

export const executeCurrentAction = defineAction<
  { tabId?: string },
  { rowCount: number; durationMs: number }
>({
  id: "query.execute.current",
  title: "Run current statement",
  description:
    "Execute the SQL statement at the cursor position in the active query tab.",
  category: "query",
  inputSchema: z.object({ tabId: z.string().optional() }),
  risk: "read",

  availability(ctx) {
    const tabId = ctx.tabId;
    const tab = tabId
      ? undefined // would need to look up tab — simplified here
      : getActiveQueryTab();
    const hasSql = tab ? tab.data.sql.trim().length > 0 : true;
    if (!ctx.connectionId)
      return { status: "unavailable", reason: "connection_required" };
    if (!hasSql) return { status: "unavailable", reason: "sql_required" };
    return { status: "available" };
  },

  async execute(input, ctx): Promise<ActionResult<{ rowCount: number; durationMs: number }>> {
    const connErr = requireConnection(ctx);
    if (connErr) return connErr as ActionResult<{ rowCount: number; durationMs: number }>;

    const tab = getActiveQueryTab();
    const sql = tab?.data.sql ?? "";
    const sqlErr = requireSql(sql);
    if (sqlErr) return sqlErr as ActionResult<{ rowCount: number; durationMs: number }>;

    const executionId = crypto.randomUUID();
    const service = createQueryService();
    const result = await service.execute(
      ctx.connectionId!,
      sql,
      executionId,
      ctx.database,
      ctx.schema,
    );

    return {
      status: "success",
      data: { rowCount: result.rowCount, durationMs: result.durationMs },
      executionId,
      effects: [
        {
          type: "query.result.updated",
          tabId: ctx.tabId,
          rowCount: result.rowCount,
        },
      ],
    };
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

  async execute(input, ctx): Promise<ActionResult<{ rowCount: number; durationMs: number }>> {
    const connErr = requireConnection(ctx);
    if (connErr) return connErr as ActionResult<{ rowCount: number; durationMs: number }>;
    const sqlErr = requireSql(input.sql);
    if (sqlErr) return sqlErr as ActionResult<{ rowCount: number; durationMs: number }>;

    const executionId = crypto.randomUUID();
    const service = createQueryService();
    const result = await service.execute(
      ctx.connectionId!,
      input.sql,
      executionId,
      ctx.database,
      ctx.schema,
    );

    return {
      status: "success",
      data: { rowCount: result.rowCount, durationMs: result.durationMs },
      executionId,
      effects: [
        { type: "query.result.updated", tabId: ctx.tabId, rowCount: result.rowCount },
      ],
    };
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

  async execute(input, ctx): Promise<ActionResult<{ totalDurationMs: number; statementCount: number }>> {
    const connErr = requireConnection(ctx);
    if (connErr) return connErr as ActionResult<{ totalDurationMs: number; statementCount: number }>;

    const tab = getActiveQueryTab();
    const sql = tab?.data.sql ?? "";
    const sqlErr = requireSql(sql);
    if (sqlErr) return sqlErr as ActionResult<{ totalDurationMs: number; statementCount: number }>;

    const executionId = crypto.randomUUID();
    const service = createQueryService();
    const result = await service.executeMulti(
      ctx.connectionId!,
      sql,
      executionId,
      ctx.database,
      ctx.schema,
    );

    return {
      status: "success",
      data: {
        totalDurationMs: result.totalDurationMs,
        statementCount: result.results.length,
      },
      executionId,
      effects: [
        {
          type: "query.multi_result.updated",
          tabId: ctx.tabId,
          statementCount: result.results.length,
        },
      ],
    };
  },
});

// ─── query.cancel ────────────────────────────────────────────

export const cancelQueryAction = defineAction<
  { executionId?: string },
  void
>({
  id: "query.cancel",
  title: "Cancel query",
  description: "Cancel the currently running query execution.",
  category: "query",
  inputSchema: z.object({ executionId: z.string().optional() }),
  risk: "read",

  availability(_ctx) {
    const tab = getActiveQueryTab();
    if (!tab || tab.data.status !== "running") {
      return { status: "unavailable", reason: "no_running_query" };
    }
    return { status: "available" };
  },

  async execute(input, ctx) {
    const execId =
      input.executionId ?? getActiveQueryTab()?.data.activeExecutionId;
    if (!execId) {
      return {
        status: "error",
        error: { code: "no_execution", message: "No running execution to cancel" },
      };
    }

    const service = createQueryService();
    await service.cancel(execId);

    return {
      status: "success",
      effects: [{ type: "query.cancelled", executionId: execId, tabId: ctx.tabId }],
    };
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

  async execute(input, ctx): Promise<ActionResult<{ plan: ExplainPlan }>> {
    const connErr = requireConnection(ctx);
    if (connErr) return connErr as ActionResult<{ plan: ExplainPlan }>;

    const tab = getActiveQueryTab();
    const sql = tab?.data.sql ?? "";
    const sqlErr = requireSql(sql);
    if (sqlErr) return sqlErr as ActionResult<{ plan: ExplainPlan }>;

    const service = createQueryService();
    const plan = await service.explain(ctx.connectionId!, sql);

    return {
      status: "success",
      data: { plan },
      effects: [{ type: "query.explain.updated", tabId: ctx.tabId }],
    };
  },
});

// ─── query.format ────────────────────────────────────────────

export const formatSqlAction = defineAction<
  { sql: string },
  { formattedSql: string }
>({
  id: "query.format",
  title: "Format SQL",
  description: "Format the SQL in the active editor.",
  category: "query",
  inputSchema: z.object({ sql: z.string().min(1) }),
  risk: "read",

  async execute(input) {
    // Dynamic import to avoid loading sql-formatter unless needed.
    const { format } = await import("sql-formatter");
    const formatted = format(input.sql);

    return {
      status: "success",
      data: { formattedSql: formatted },
      effects: [{ type: "query.sql.formatted" }],
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

  async execute(input, ctx) {
    const tabId = input.tabId ?? ctx.tabId;
    if (!tabId) {
      return {
        status: "error",
        error: { code: "no_tab", message: "No active query tab" },
      };
    }

    // Side-effect on workspace state (same pattern as controllers).
    const { updateTabData } = await import("@/commons/stores/workspace.store").then(
      (m) => m.useWorkspaceStore.getState(),
    );
    updateTabData(tabId, (data) => ({ ...data, sql: "" }));

    return {
      status: "success",
      effects: [{ type: "query.sql.cleared", tabId }],
    };
  },
});

// ─── query.save ──────────────────────────────────────────────

export const saveQueryAction = defineAction<
  { name: string; tabId?: string },
  { savedQueryId: string }
>({
  id: "query.save",
  title: "Save query",
  description: "Save the current SQL as a named query.",
  category: "query",
  inputSchema: z.object({
    name: z.string().min(1),
    tabId: z.string().optional(),
  }),
  risk: "read",

  async execute(input, ctx): Promise<ActionResult<{ savedQueryId: string }>> {
    const connErr = requireConnection(ctx);
    if (connErr) return connErr as ActionResult<{ savedQueryId: string }>;

    const tab = getActiveQueryTab();
    const sql = tab?.data.sql ?? "";
    const sqlErr = requireSql(sql);
    if (sqlErr) return sqlErr as ActionResult<{ savedQueryId: string }>;

    const service = createQueryService();
    const saved = await service.save(ctx.connectionId!, input.name, sql);

    return {
      status: "success",
      data: { savedQueryId: saved.id },
      effects: [{ type: "query.saved", savedQueryId: saved.id }],
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

  async execute(_input, _ctx) {
    const tab = getActiveQueryTab();
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

  async execute(_input, _ctx) {
    const tab = getActiveQueryTab();
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

  async execute(_input, _ctx) {
    const tab = getActiveQueryTab();
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
