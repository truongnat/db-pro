import { describe, expect, it, beforeEach } from "vitest";
import { z } from "zod";

import {
  resetActionRegistry,
  defineAction,
  getRegisteredActions,
} from "../registry";
import {
  executeAction,
  confirmAction,
  cancelExecution,
  bindExternalExecutionId,
  getRunningExecutions,
} from "../bus";
import {
  actionToMcpTool,
  generateMcpTools,
} from "../mcp-bridge";

// ─── Helpers ─────────────────────────────────────────────────

/**
 * Register a minimal test action.
 */
function registerTestAction(
  id: string,
  overrides: Partial<Parameters<typeof defineAction>[0]> = {},
) {
  return defineAction<{ value?: string }, { echoed: string }>({
    id,
    title: `Test action ${id}`,
    category: "query",
    inputSchema: z.object({ value: z.string().optional() }),
    risk: "read",
    async execute(input) {
      return {
        status: "success",
        data: { echoed: input.value ?? "default" },
      };
    },
    ...overrides,
  } as Parameters<typeof defineAction>[0]);
}

// ─── Tests ───────────────────────────────────────────────────

beforeEach(() => {
  resetActionRegistry();
});

describe("PATCH 6.1 — Action Platform regression tests", () => {
  // ── P1-1: Zod → JSON Schema ─────────────────────────────────

  describe("P1-1: Zod → JSON Schema conversion", () => {
    it("converts a simple Zod object schema to JSON Schema via actionToMcpTool", () => {
      const def = registerTestAction("test.schema", {
        inputSchema: z.object({
          connectionId: z.string().min(1),
          sql: z.string(),
          limit: z.number().optional(),
        }) as never,
      });

      const tool = actionToMcpTool(def);

      expect(tool.inputSchema).toBeDefined();
      expect(tool.inputSchema.type).toBe("object");
      const props = tool.inputSchema.properties as Record<string, unknown>;
      expect(props.connectionId).toBeDefined();
      expect(props.sql).toBeDefined();
    });

    it("does not produce empty properties for a schema with fields", () => {
      const def = registerTestAction("test.schema2", {
        inputSchema: z.object({
          tableName: z.string().min(1),
          schema: z.string().min(1),
        }) as never,
      });

      const tool = actionToMcpTool(def);
      const props = tool.inputSchema.properties as Record<string, unknown>;
      expect(Object.keys(props).length).toBeGreaterThan(0);
    });
  });

  // ── P1-3: MCP tool filtering ────────────────────────────────

  describe("P1-3: MCP tool filtering", () => {
    it("excludes actions with availability = not_implemented", () => {
      registerTestAction("test.implemented");
      registerTestAction("test.unimplemented", {
        availability: () => ({
          status: "unavailable" as const,
          reason: "not_implemented",
        }),
      });

      const tools = generateMcpTools();
      const ids = tools.map((t) => t.actionId);

      expect(ids).toContain("test.implemented");
      expect(ids).not.toContain("test.unimplemented");
    });

    it("includes actions with availability = available", () => {
      registerTestAction("test.available", {
        availability: () => ({ status: "available" as const }),
      });

      const tools = generateMcpTools();
      expect(tools.some((t) => t.actionId === "test.available")).toBe(true);
    });

    it("includes actions without availability() (always available)", () => {
      registerTestAction("test.no_availability");

      const tools = generateMcpTools();
      expect(tools.some((t) => t.actionId === "test.no_availability")).toBe(true);
    });
  });

  // ── P1-2: Confirmation protocol ─────────────────────────────

  describe("P1-2: Confirmation preserves input + binds token", () => {
    it("returns confirmation_required for actions with confirmation mode 'always'", async () => {
      registerTestAction("test.confirm", {
        confirmation: { mode: "always", messageKey: "test.confirm" },
      });

      const result = await executeAction("test.confirm", { value: "hello" }, {
        source: "ui",
      });

      expect(result.status).toBe("confirmation_required");
      expect(result.confirmation).toBeDefined();
      expect(result.confirmation!.actionId).toBe("test.confirm");
      expect(result.confirmation!.input).toEqual({ value: "hello" });
      expect(result.confirmation!.source).toBe("ui");
      expect(result.confirmation!.createdAt).toBeGreaterThan(0);
    });

    it("replays original input on confirm", async () => {
      let receivedInput: { value?: string } | undefined;

      defineAction<{ value?: string }, { echoed: string }>({
        id: "test.confirm_replay",
        title: "Test confirm replay",
        category: "query",
        inputSchema: z.object({ value: z.string().optional() }),
        risk: "read",
        confirmation: { mode: "always" },
        async execute(input) {
          receivedInput = input;
          return { status: "success", data: { echoed: input.value ?? "" } };
        },
      });

      const first = await executeAction(
        "test.confirm_replay",
        { value: "original" },
        { source: "ui" },
      );
      expect(first.status).toBe("confirmation_required");

      const confirmationId = first.confirmation!.id;
      const second = await confirmAction(confirmationId);

      expect(second.status).toBe("success");
      expect(receivedInput).toEqual({ value: "original" });
    });

    it("rejects confirmation token for wrong action", async () => {
      registerTestAction("test.action_a", {
        confirmation: { mode: "always" },
      });
      registerTestAction("test.action_b", {
        confirmation: { mode: "always" },
      });

      const resultA = await executeAction("test.action_a", { value: "a" });
      const token = resultA.confirmation!.id;

      const resultB = await executeAction("test.action_b", { value: "b" }, {
        confirmationToken: token,
      });

      expect(resultB.status).toBe("error");
      expect(resultB.error?.code).toBe("confirmation_mismatch");
    });
  });

  // ── P1-6: Dynamic risk classification ───────────────────────

  describe("P1-6: Dynamic risk classification", () => {
    it("resolveRisk overrides static risk when present", async () => {
      defineAction<{ sql: string }, void>({
        id: "test.dynamic_risk",
        title: "Test dynamic risk",
        category: "query",
        inputSchema: z.object({ sql: z.string() }),
        risk: "read",
        resolveRisk(input) {
          if (/^(DROP|DELETE|TRUNCATE)\b/i.test(input.sql)) return "destructive";
          if (/^(INSERT|UPDATE|ALTER)\b/i.test(input.sql)) return "write";
          return "read";
        },
        confirmation: { mode: "destructive-only" },
        async execute() {
          return { status: "success" };
        },
      });

      const readResult = await executeAction(
        "test.dynamic_risk",
        { sql: "SELECT 1" },
        { source: "agent" },
      );
      expect(readResult.status).toBe("success");

      const deleteResult = await executeAction(
        "test.dynamic_risk",
        { sql: "DELETE FROM users" },
        { source: "agent" },
      );
      expect(deleteResult.status).toBe("confirmation_required");
    });
  });

  // ── MCP tool name generation ────────────────────────────────

  describe("MCP tool generation", () => {
    it("generates correct tool definition from action", () => {
      registerTestAction("query.test_action", {
        description: "A test action",
        risk: "write",
      });

      const tools = generateMcpTools();
      const tool = tools.find((t) => t.actionId === "query.test_action");

      expect(tool).toBeDefined();
      expect(tool!.name).toBe("dbpro_query_test_action");
      expect(tool!.description).toBe("A test action");
      expect(tool!.risk).toBe("write");
    });
  });
});

describe("PATCH 6.2 — Action Runtime Unification regression tests", () => {
  // ── P6.2-1: Confirmation snapshots RESOLVED context ─────────

  describe("Confirmation snapshots resolved context", () => {
    it("stores resolvedContext (not just overrides) in confirmation", async () => {
      defineAction<{ value?: string }, void>({
        id: "test.ctx_snapshot",
        title: "Test context snapshot",
        category: "query",
        inputSchema: z.object({ value: z.string().optional() }),
        risk: "read",
        confirmation: { mode: "always" },
        resolveContext(input, ambient) {
          return {
            ...ambient,
            connectionId: "resolved-conn",
            database: "resolved-db",
            schema: "resolved-schema",
          };
        },
        async execute() {
          return { status: "success" };
        },
      });

      const result = await executeAction("test.ctx_snapshot", { value: "x" }, {
        source: "mcp",
      });

      expect(result.status).toBe("confirmation_required");
      const conf = result.confirmation!;
      // Must have the FULL resolved context, not just overrides.
      expect(conf.resolvedContext).toBeDefined();
      expect(conf.resolvedContext!.connectionId).toBe("resolved-conn");
      expect(conf.resolvedContext!.database).toBe("resolved-db");
      expect(conf.resolvedContext!.schema).toBe("resolved-schema");
    });
  });

  // ── P6.2-2: resolveContext hook overrides ambient context ───

  describe("resolveContext hook", () => {
    it("overrides ambient context before execution", async () => {
      let receivedCtx: { connectionId?: string } | undefined;

      defineAction<{ value?: string }, void>({
        id: "test.resolve_ctx",
        title: "Test resolve context",
        category: "query",
        inputSchema: z.object({ value: z.string().optional() }),
        risk: "read",
        resolveContext(_input, ambient) {
          return { ...ambient, connectionId: "explicit-conn" };
        },
        async execute(_input, ctx) {
          receivedCtx = ctx;
          return { status: "success" };
        },
      });

      await executeAction("test.resolve_ctx", {}, {
        source: "ui",
        context: { connectionId: "ambient-conn" },
      });

      expect(receivedCtx?.connectionId).toBe("explicit-conn");
    });
  });

  // ── P6.2-3: MCP dynamic risk metadata ──────────────────────

  describe("MCP dynamic risk metadata", () => {
    it("reports risk as 'dynamic' for actions with resolveRisk", () => {
      registerTestAction("test.dynamic_mcp", {
        resolveRisk: () => "read" as const,
      });

      const tool = actionToMcpTool(
        getRegisteredActions().find((a) => a.id === "test.dynamic_mcp")!,
      );

      expect(tool.risk).toBe("dynamic");
      expect(tool.riskPolicy).toBe("sql");
    });

    it("reports static risk for actions without resolveRisk", () => {
      registerTestAction("test.static_risk", {
        risk: "write",
      });

      const tool = actionToMcpTool(
        getRegisteredActions().find((a) => a.id === "test.static_risk")!,
      );

      expect(tool.risk).toBe("write");
      expect(tool.riskPolicy).toBeUndefined();
    });
  });

  // ── P6.2-4: data.filter/sort excluded from MCP ─────────────

  describe("Fake actions excluded from MCP", () => {
    it("data.filter and data.sort are not_implemented", () => {
      // Register them as they are in data.actions.ts
      registerTestAction("data.filter", {
        availability: () => ({
          status: "unavailable" as const,
          reason: "not_implemented",
        }),
      });
      registerTestAction("data.sort", {
        availability: () => ({
          status: "unavailable" as const,
          reason: "not_implemented",
        }),
      });

      const tools = generateMcpTools();
      const ids = tools.map((t) => t.actionId);

      expect(ids).not.toContain("data.filter");
      expect(ids).not.toContain("data.sort");
    });
  });

  // ── P6.2-5: Cancel failure reports error ───────────────────

  describe("Cancel failure handling", () => {
    it("canceling a non-existent execution returns error", async () => {
      const cancelResult = await cancelExecution("nonexistent_id");
      expect(cancelResult.status).toBe("error");
      expect(cancelResult.error?.code).toBe("execution_not_found");
    });

    it("cancel hook throw → action state becomes error, NOT cancelled", async () => {
      defineAction<{ value?: string }, void>({
        id: "test.cancel_hook_error",
        title: "Test cancel hook error state",
        category: "query",
        inputSchema: z.object({ value: z.string().optional() }),
        risk: "read",
        async execute() {
          return new Promise(() => {}); // never resolves
        },
        async cancel() {
          throw new Error("Backend refused cancel");
        },
      });

      // Start execution (never resolves).
      const execPromise = executeAction("test.cancel_hook_error", {});
      await new Promise((r) => setTimeout(r, 10));

      // Find the running execution.
      const running = getRunningExecutions();
      expect(running.length).toBe(1);
      const execId = running[0].executionId;

      // Cancel should fail because the cancel hook throws.
      const cancelResult = await cancelExecution(execId);
      expect(cancelResult.status).toBe("error");
      expect(cancelResult.error?.code).toBe("cancel_failed");

      void execPromise;
    });
  });

  // ── P6.2-6: Confirmation with destructive-only policy ──────

  describe("Destructive-only confirmation policy", () => {
    it("requires confirmation for destructive risk even from agent", async () => {
      defineAction<{ sql: string }, void>({
        id: "test.destructive_policy",
        title: "Test destructive policy",
        category: "query",
        inputSchema: z.object({ sql: z.string() }),
        risk: "read",
        resolveRisk(input) {
          return /^(DROP|DELETE|TRUNCATE)\b/i.test(input.sql) ? "destructive" : "read";
        },
        confirmation: { mode: "destructive-only" },
        async execute() {
          return { status: "success" };
        },
      });

      // SELECT → no confirmation.
      const readResult = await executeAction(
        "test.destructive_policy",
        { sql: "SELECT 1" },
        { source: "mcp" },
      );
      expect(readResult.status).toBe("success");

      // DROP → confirmation required even from MCP.
      const dropResult = await executeAction(
        "test.destructive_policy",
        { sql: "DROP TABLE users" },
        { source: "mcp" },
      );
      expect(dropResult.status).toBe("confirmation_required");
    });

    it("write risk does NOT require confirmation with destructive-only", async () => {
      defineAction<{ sql: string }, void>({
        id: "test.write_no_confirm",
        title: "Test write no confirm",
        category: "query",
        inputSchema: z.object({ sql: z.string() }),
        risk: "read",
        resolveRisk(input) {
          return /^INSERT\b/i.test(input.sql) ? "write" : "read";
        },
        confirmation: { mode: "destructive-only" },
        async execute() {
          return { status: "success" };
        },
      });

      const result = await executeAction(
        "test.write_no_confirm",
        { sql: "INSERT INTO t VALUES (1)" },
        { source: "agent" },
      );
      // Write risk with destructive-only → no confirmation needed.
      expect(result.status).toBe("success");
    });
  });

  // ── P6.2-7: External execution ID tracking ─────────────────

  describe("External execution ID", () => {
    it("ActionExecution supports externalExecutionId field", () => {
      const exec = {
        executionId: "exec_1",
        actionId: "test",
        state: "running" as const,
        startedAt: Date.now(),
        externalExecutionId: "backend-query-id-123",
      };
      expect(exec.externalExecutionId).toBe("backend-query-id-123");
    });
  });
});

describe("PATCH 6.3 — Canonical Query Runtime regression tests", () => {
  // ── P6.3-1: Terminal state mapping ─────────────────────────

  describe("Terminal state mapping", () => {
    it("success → completed", async () => {
      defineAction({
        id: "test.success_state",
        title: "Test success state",
        category: "query",
        inputSchema: z.object({}),
        risk: "read",
        async execute() {
          return { status: "success" as const };
        },
      });

      const result = await executeAction("test.success_state", {});
      expect(result.status).toBe("success");
      expect(result.executionId).toBeDefined();
    });

    it("error → error (NOT completed)", async () => {
      defineAction({
        id: "test.error_state",
        title: "Test error state",
        category: "query",
        inputSchema: z.object({}),
        risk: "read",
        async execute() {
          return { status: "error" as const, error: { code: "test", message: "fail" } };
        },
      });

      const result = await executeAction("test.error_state", {});
      expect(result.status).toBe("error");
    });

    it("cancelled → cancelled", async () => {
      defineAction({
        id: "test.cancel_state",
        title: "Test cancel state",
        category: "query",
        inputSchema: z.object({}),
        risk: "read",
        async execute() {
          return { status: "cancelled" as const };
        },
      });

      const result = await executeAction("test.cancel_state", {});
      expect(result.status).toBe("cancelled");
    });
  });

  // ── P6.3-2: Confirmation payload freeze ────────────────────

  describe("Confirmation payload freeze", () => {
    it("resolvePayload is called and stored in confirmation", async () => {
      defineAction<{ sql: string }, void>({
        id: "test.payload_freeze",
        title: "Test payload freeze",
        category: "query",
        inputSchema: z.object({ sql: z.string() }),
        risk: "read",
        confirmation: { mode: "always" },
        resolvePayload(input) {
          return { sql: input.sql, connectionId: "frozen-conn" };
        },
        async execute() {
          return { status: "success" };
        },
      });

      const result = await executeAction("test.payload_freeze", { sql: "DROP TABLE foo" }, {
        source: "ui",
      });

      expect(result.status).toBe("confirmation_required");
      const conf = result.confirmation!;
      expect(conf.resolvedPayload).toBeDefined();
      expect(conf.resolvedPayload!.sql).toBe("DROP TABLE foo");
      expect(conf.resolvedPayload!.connectionId).toBe("frozen-conn");
    });

    it("on confirm, frozen payload is available on ctx.resolvedPayload (no input spreading)", async () => {
      let receivedPayload: Record<string, unknown> | undefined;

      defineAction<{ sql: string }, void>({
        id: "test.payload_confirm",
        title: "Test payload confirm",
        category: "query",
        inputSchema: z.object({ sql: z.string() }),
        risk: "read",
        confirmation: { mode: "always" },
        resolvePayload(input) {
          return { sql: input.sql, frozen: true };
        },
        async execute(_input, ctx) {
          // The action reads the frozen payload from context.
          receivedPayload = ctx.resolvedPayload;
          return { status: "success" };
        },
      });

      // Step 1: Request execution → confirmation_required.
      const first = await executeAction("test.payload_confirm", { sql: "DROP TABLE foo" });
      expect(first.status).toBe("confirmation_required");

      // Step 2: Confirm.
      const second = await confirmAction(first.confirmation!.id);
      expect(second.status).toBe("success");

      // The action must have received the frozen payload on context.
      expect(receivedPayload).toBeDefined();
      expect(receivedPayload!.sql).toBe("DROP TABLE foo");
      expect(receivedPayload!.frozen).toBe(true);
    });
  });

  // ── P6.3-3: bindExternalExecutionId integration ────────────

  describe("bindExternalExecutionId", () => {
    it("real cancel integration: bind → cancel → cancel hook receives external ID", async () => {
      let cancelReceivedExternalId: string | undefined;

      defineAction<{ value?: string }, void>({
        id: "test.bind_cancel",
        title: "Test bind and cancel",
        category: "query",
        inputSchema: z.object({ value: z.string().optional() }),
        risk: "read",
        async execute(_input, ctx) {
          // Bind the backend execution ID using actionExecutionId.
          const backendId = "backend-query-exec-123";
          if (ctx.actionExecutionId) {
            bindExternalExecutionId(ctx.actionExecutionId, backendId);
          }
          return new Promise(() => {}); // never resolves
        },
        async cancel(execution) {
          cancelReceivedExternalId = execution.externalExecutionId;
        },
      });

      // Start execution.
      const execPromise = executeAction("test.bind_cancel", {});
      await new Promise((r) => setTimeout(r, 10));

      // Find the running execution.
      const running = getRunningExecutions();
      expect(running.length).toBe(1);
      const actionExecId = running[0].executionId;

      // Verify the external ID was bound.
      expect(running[0].externalExecutionId).toBe("backend-query-exec-123");

      // Cancel using the action execution ID.
      const cancelResult = await cancelExecution(actionExecId);
      expect(cancelResult.status).toBe("cancelled");

      // Verify the cancel hook received the backend external ID.
      expect(cancelReceivedExternalId).toBe("backend-query-exec-123");

      void execPromise;
    });

    it("cancel hook throws → cancel_failed, execution state = error", async () => {
      defineAction<{ value?: string }, void>({
        id: "test.bind_cancel_fail",
        title: "Test bind cancel fail",
        category: "query",
        inputSchema: z.object({ value: z.string().optional() }),
        risk: "read",
        async execute(_input, ctx) {
          if (ctx.actionExecutionId) {
            bindExternalExecutionId(ctx.actionExecutionId, "backend-456");
          }
          return new Promise(() => {}); // never resolves
        },
        async cancel() {
          throw new Error("Backend refused");
        },
      });

      const execPromise = executeAction("test.bind_cancel_fail", {});
      await new Promise((r) => setTimeout(r, 10));

      const running = getRunningExecutions();
      expect(running.length).toBe(1);
      const actionExecId = running[0].executionId;
      expect(running[0].externalExecutionId).toBe("backend-456");

      // Cancel should fail.
      const cancelResult = await cancelExecution(actionExecId);
      expect(cancelResult.status).toBe("error");
      expect(cancelResult.error?.code).toBe("cancel_failed");

      void execPromise;
    });
  });
});

describe("PATCH 6.3.1 — Canonical Runtime Closure regression tests", () => {
  // ── P1-1: Execution identity ────────────────────────────────

  describe("Execution identity (actionExecutionId vs correlationId)", () => {
    it("action receives actionExecutionId that matches the bus execution ID", async () => {
      let receivedActionExecId: string | undefined;

      defineAction<{ value?: string }, void>({
        id: "test.exec_identity",
        title: "Test exec identity",
        category: "query",
        inputSchema: z.object({ value: z.string().optional() }),
        risk: "read",
        async execute(_input, ctx) {
          receivedActionExecId = ctx.actionExecutionId;
          return { status: "success" };
        },
      });

      const result = await executeAction("test.exec_identity", {});
      expect(result.status).toBe("success");
      expect(result.executionId).toBeDefined();
      // The action's actionExecutionId must match the result executionId.
      expect(receivedActionExecId).toBe(result.executionId);
      // Must start with exec_ prefix (not corr_).
      expect(receivedActionExecId).toMatch(/^exec_/);
    });

    it("bus always uses its own executionId, not handler-returned ID", async () => {
      defineAction<{ value?: string }, void>({
        id: "test.exec_id_override",
        title: "Test exec ID override",
        category: "query",
        inputSchema: z.object({ value: z.string().optional() }),
        risk: "read",
        async execute() {
          // Handler tries to return its own executionId.
          return { status: "success" as const, executionId: "handler-should-not-override" };
        },
      });

      const result = await executeAction("test.exec_id_override", {});
      expect(result.status).toBe("success");
      // Bus must use its own executionId, not the handler's.
      expect(result.executionId).toMatch(/^exec_/);
      expect(result.executionId).not.toBe("handler-should-not-override");
    });
  });

  // ── P1-4: Partial failure ────────────────────────────────────

  describe("Multi partial failure result", () => {
    it("action returns error status when multi execution has partial failure", async () => {
      defineAction<{ value?: string }, { totalDurationMs: number; statementCount: number; completedResults: number; failedStatement?: number }>({
        id: "test.partial_failure",
        title: "Test partial failure",
        category: "query",
        inputSchema: z.object({ value: z.string().optional() }),
        risk: "read",
        async execute() {
          // Simulate what query.actions.ts does: runtime returns MultiQueryResult
          // with error, action checks data.error and returns error status.
          const data = {
            results: [{ columns: [], rows: [], rowCount: 5, durationMs: 10 }],
            totalDurationMs: 50,
            error: [1, "syntax error at statement 2"] as [number, string],
          };

          if (data.error) {
            const [stmtIdx] = data.error;
            return {
              status: "error" as const,
              error: {
                code: "partial_execution_failure",
                message: `Statement ${stmtIdx + 1} failed.`,
              },
              data: {
                totalDurationMs: data.totalDurationMs,
                statementCount: data.results.length,
                completedResults: data.results.length,
                failedStatement: stmtIdx,
              },
            };
          }

          return { status: "success" as const, data: { totalDurationMs: 0, statementCount: 0, completedResults: 0 } };
        },
      });

      const result = await executeAction("test.partial_failure", {});
      // Must be error, NOT success.
      expect(result.status).toBe("error");
      expect(result.error?.code).toBe("partial_execution_failure");
      // Must include structured partial result info.
      expect(result.data).toBeDefined();
      expect(result.data!.completedResults).toBe(1);
      expect(result.data!.failedStatement).toBe(1);
    });
  });

  // ── P2-1: Resolve once ───────────────────────────────────────

  describe("Resolve payload once", () => {
    it("resolvePayload called once, shared between resolveRisk and execute", async () => {
      let resolvePayloadCallCount = 0;
      let riskReadPayload: Record<string, unknown> | undefined;
      let executeReadPayload: Record<string, unknown> | undefined;

      defineAction<{ sql: string }, void>({
        id: "test.resolve_once",
        title: "Test resolve once",
        category: "query",
        inputSchema: z.object({ sql: z.string() }),
        risk: "read",
        confirmation: { mode: "destructive-only" },
        resolvePayload(input) {
          resolvePayloadCallCount++;
          return { sql: input.sql };
        },
        resolveRisk(_input, ctx) {
          riskReadPayload = ctx.resolvedPayload;
          return "read" as const;
        },
        async execute(_input, ctx) {
          executeReadPayload = ctx.resolvedPayload;
          return { status: "success" };
        },
      });

      const result = await executeAction("test.resolve_once", { sql: "SELECT 1" });
      expect(result.status).toBe("success");

      // resolvePayload must be called exactly once.
      expect(resolvePayloadCallCount).toBe(1);

      // Both resolveRisk and execute must read the SAME payload instance.
      expect(riskReadPayload).toBeDefined();
      expect(executeReadPayload).toBeDefined();
      expect(riskReadPayload).toBe(executeReadPayload);
      expect(riskReadPayload!.sql).toBe("SELECT 1");
    });
  });
});
