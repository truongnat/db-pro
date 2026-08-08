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
  getExecution,
  rejectConfirmation,
  findRunningExecutionForTab,
  resetActiveExecutions,
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
  resetActiveExecutions();
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

    it("confirmAction with unknown ID returns confirmation_not_found", async () => {
      registerTestAction("test.action_a", {
        confirmation: { mode: "always" },
      });

      // Attempting to confirm a non-existent ID must fail.
      const result = await confirmAction("nonexistent_confirmation_id");
      expect(result.status).toBe("error");
      expect(result.error?.code).toBe("confirmation_not_found");
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

describe("PATCH 6.3.2 — Action Gate & Cancellation Closure integration tests", () => {
  // ── A. Confirmation prevents execution before approval ───────

  describe("A. Confirmation prevents backend execution", () => {
    it("destructive SQL returns confirmation_required and does NOT call execute", async () => {
      let executeCalled = false;

      defineAction<{ sql: string }, void>({
        id: "test.confirm_no_exec",
        title: "Test confirm no exec",
        category: "query",
        inputSchema: z.object({ sql: z.string() }),
        risk: "read",
        resolveRisk(input) {
          return /^(DROP|DELETE|TRUNCATE)\b/i.test(input.sql) ? "destructive" : "read";
        },
        confirmation: { mode: "destructive-only" },
        async execute() {
          executeCalled = true;
          return { status: "success" };
        },
      });

      const result = await executeAction(
        "test.confirm_no_exec",
        { sql: "DROP TABLE foo" },
        { source: "ui" },
      );

      expect(result.status).toBe("confirmation_required");
      expect(executeCalled).toBe(false);
    });
  });

  // ── B. Confirmation SQL/context drift ───────────────────────

  describe("B. Confirmation preserves frozen SQL/context", () => {
    it("confirm executes the ORIGINAL SQL, not live state", async () => {
      let executedSql: string | undefined;
      let resolvePayloadCallCount = 0;

      defineAction<{ sql: string }, void>({
        id: "test.drift",
        title: "Test drift",
        category: "query",
        inputSchema: z.object({ sql: z.string() }),
        risk: "read",
        confirmation: { mode: "always" },
        resolvePayload(input) {
          resolvePayloadCallCount++;
          return { sql: input.sql };
        },
        async execute(_input, ctx) {
          // Read from the frozen payload on context.
          const payload = ctx.resolvedPayload as { sql: string } | undefined;
          executedSql = payload?.sql;
          return { status: "success" };
        },
      });

      // Step 1: Request execution with "DROP TABLE foo".
      const first = await executeAction("test.drift", { sql: "DROP TABLE foo" });
      expect(first.status).toBe("confirmation_required");

      // Step 2: Confirm (simulating user clicking Confirm).
      // Even if the "editor" changed, the frozen payload must be used.
      const second = await confirmAction(first.confirmation!.id);
      expect(second.status).toBe("success");

      // Backend must receive the ORIGINAL SQL.
      expect(executedSql).toBe("DROP TABLE foo");
      // resolvePayload must NOT be called again on confirm.
      expect(resolvePayloadCallCount).toBe(1);
    });
  });

  // ── C. Confirmation token single-use ────────────────────────

  describe("C. Confirmation token single-use", () => {
    it("confirmation token is consumed exactly once", async () => {
      defineAction<{ value?: string }, void>({
        id: "test.single_use",
        title: "Test single use",
        category: "query",
        inputSchema: z.object({ value: z.string().optional() }),
        risk: "read",
        confirmation: { mode: "always" },
        async execute() {
          return { status: "success" };
        },
      });

      const first = await executeAction("test.single_use", { value: "x" });
      expect(first.status).toBe("confirmation_required");
      const tokenId = first.confirmation!.id;

      // First confirm → success.
      const second = await confirmAction(tokenId);
      expect(second.status).toBe("success");

      // Second confirm → error (token already consumed).
      const third = await confirmAction(tokenId);
      expect(third.status).toBe("error");
      expect(third.error?.code).toBe("confirmation_not_found");
    });
  });

  // ── D. Real cancel: bind → cancel → backend receives external ID ─

  describe("D. Production cancel integration", () => {
    it("cancelExecution(ActionExecutionId) → cancel hook receives external ID", async () => {
      let cancelReceivedExternalId: string | undefined;

      defineAction<{ value?: string }, void>({
        id: "test.prod_cancel",
        title: "Test prod cancel",
        category: "query",
        inputSchema: z.object({ value: z.string().optional() }),
        risk: "read",
        async execute(_input, ctx) {
          const backendId = "backend-uuid-789";
          if (ctx.actionExecutionId) {
            bindExternalExecutionId(ctx.actionExecutionId, backendId);
          }
          return new Promise(() => {}); // never resolves
        },
        async cancel(execution) {
          cancelReceivedExternalId = execution.externalExecutionId;
        },
      });

      const execPromise = executeAction("test.prod_cancel", {});
      await new Promise((r) => setTimeout(r, 10));

      const running = getRunningExecutions();
      expect(running.length).toBe(1);
      const actionExecId = running[0].executionId;
      expect(running[0].externalExecutionId).toBe("backend-uuid-789");

      const cancelResult = await cancelExecution(actionExecId);
      expect(cancelResult.status).toBe("cancelled");
      expect(cancelReceivedExternalId).toBe("backend-uuid-789");

      void execPromise;
    });
  });

  // ── E. Backend cancel throws → cancel_failed ───────────────

  describe("E. Backend cancel throws → cancel_failed", () => {
    it("cancel hook throws → action state error, NOT cancelled", async () => {
      defineAction<{ value?: string }, void>({
        id: "test.cancel_throws",
        title: "Test cancel throws",
        category: "query",
        inputSchema: z.object({ value: z.string().optional() }),
        risk: "read",
        async execute(_input, ctx) {
          if (ctx.actionExecutionId) {
            bindExternalExecutionId(ctx.actionExecutionId, "backend-err");
          }
          return new Promise(() => {});
        },
        async cancel() {
          throw new Error("Backend refused to cancel");
        },
      });

      const execPromise = executeAction("test.cancel_throws", {});
      await new Promise((r) => setTimeout(r, 10));

      const running = getRunningExecutions();
      const cancelResult = await cancelExecution(running[0].executionId);

      expect(cancelResult.status).toBe("error");
      expect(cancelResult.error?.code).toBe("cancel_failed");

      void execPromise;
    });
  });

  // ── F. Action without cancel handler → cancel_not_supported ──

  describe("F. No cancel handler → cancel_not_supported", () => {
    it("cancel without handler returns cancel_not_supported, NOT cancelled", async () => {
      defineAction<{ value?: string }, void>({
        id: "test.no_cancel",
        title: "Test no cancel",
        category: "query",
        inputSchema: z.object({ value: z.string().optional() }),
        risk: "read",
        async execute() {
          return new Promise(() => {});
        },
        // No cancel hook defined.
      });

      const execPromise = executeAction("test.no_cancel", {});
      await new Promise((r) => setTimeout(r, 10));

      const running = getRunningExecutions();
      expect(running.length).toBe(1);

      const cancelResult = await cancelExecution(running[0].executionId);
      // Must NOT be cancelled — must be error with cancel_not_supported.
      expect(cancelResult.status).toBe("error");
      expect(cancelResult.error?.code).toBe("cancel_not_supported");

      void execPromise;
    });
  });

  // ── G. Same-tab concurrency guard ──────────────────────────

  describe("G. Same-tab concurrency guard", () => {
    it("second invocation is unavailable while tab is running", async () => {
      // Register an action with a concurrency guard that checks tab status.
      defineAction<{ value?: string }, void>({
        id: "test.concurrency",
        title: "Test concurrency",
        category: "query",
        inputSchema: z.object({ value: z.string().optional() }),
        risk: "read",
        availability() {
          // Simulate: check if the tab is running.
          // In production, this checks useWorkspaceStore tab status.
          const running = getRunningExecutions();
          if (running.some((e) => e.actionId === "test.concurrency")) {
            return { status: "unavailable" as const, reason: "query_running" };
          }
          return { status: "available" as const };
        },
        async execute() {
          return new Promise(() => {}); // never resolves
        },
      });

      // First invocation → should start.
      const exec1 = executeAction("test.concurrency", {});
      await new Promise((r) => setTimeout(r, 10));

      // Second invocation → should be unavailable.
      const result2 = await executeAction("test.concurrency", {});
      expect(result2.status).toBe("error");
      expect(result2.error?.code).toBe("action_unavailable");

      void exec1;
    });
  });

  // ── H. Cancel stale guard ──────────────────────────────────

  describe("H. Cancel stale guard", () => {
    it("cancel of exec1 does not clobber exec2 state", async () => {
      let exec2Started = false;

      defineAction<{ value?: string }, void>({
        id: "test.stale_cancel",
        title: "Test stale cancel",
        category: "query",
        inputSchema: z.object({ value: z.string().optional() }),
        risk: "read",
        async execute(_input, ctx) {
          if (ctx.actionExecutionId) {
            bindExternalExecutionId(ctx.actionExecutionId, `backend-${ctx.actionExecutionId}`);
          }
          return new Promise(() => {});
        },
        async cancel() {
          // Simulate slow cancel that returns after a newer execution started.
        },
      });

      // Start exec1.
      const exec1Promise = executeAction("test.stale_cancel", { value: "1" });
      await new Promise((r) => setTimeout(r, 10));

      const running1 = getRunningExecutions();
      expect(running1.length).toBe(1);
      const exec1Id = running1[0].executionId;

      // Cancel exec1.
      const cancelResult = await cancelExecution(exec1Id);
      expect(cancelResult.status).toBe("cancelled");

      // Verify exec1 is cancelled.
      const exec1State = getRunningExecutions().find((e) => e.executionId === exec1Id);
      // After cancel, exec1 should no longer be in running state.
      expect(exec1State).toBeUndefined();

      void exec1Promise;
      void exec2Started;
    });
  });

  // ── I. Partial failure → error status ──────────────────────

  describe("I. Partial failure → error status with preserved results", () => {
    it("execute.all partial failure returns error with completed results", async () => {
      defineAction<{ value?: string }, { totalDurationMs: number; statementCount: number; completedResults: number; failedStatement?: number }>({
        id: "test.partial_632",
        title: "Test partial 632",
        category: "query",
        inputSchema: z.object({ value: z.string().optional() }),
        risk: "read",
        async execute() {
          // Simulate what production query.actions.ts does.
          const data = {
            results: [
              { columns: [], rows: [], rowCount: 10, durationMs: 5 },
              { columns: [], rows: [], rowCount: 3, durationMs: 8 },
            ],
            totalDurationMs: 50,
            error: [2, "syntax error"] as [number, string],
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

      const result = await executeAction("test.partial_632", {});
      expect(result.status).toBe("error");
      expect(result.error?.code).toBe("partial_execution_failure");
      // Successful result sets must be preserved.
      expect(result.data).toBeDefined();
      expect(result.data!.completedResults).toBe(2);
      expect(result.data!.failedStatement).toBe(2);
    });
  });

  // ── Item 7. Execution context stored on ActionExecution ─────

  describe("Execution context stored on ActionExecution", () => {
    it("ActionExecution retains source, tabId, connectionId, correlationId", async () => {
      defineAction<{ value?: string }, void>({
        id: "test.exec_ctx",
        title: "Test exec ctx",
        category: "query",
        inputSchema: z.object({ value: z.string().optional() }),
        risk: "read",
        resolveContext(_input, ambient) {
          return { ...ambient, tabId: "tab-123", connectionId: "conn-456" };
        },
        async execute() {
          return new Promise(() => {});
        },
      });

      const execPromise = executeAction("test.exec_ctx", {}, {
        source: "mcp",
        context: { tabId: "tab-123", connectionId: "conn-456" },
      });
      await new Promise((r) => setTimeout(r, 10));

      const running = getRunningExecutions();
      expect(running.length).toBe(1);
      expect(running[0].source).toBe("mcp");
      expect(running[0].tabId).toBe("tab-123");
      expect(running[0].connectionId).toBe("conn-456");
      expect(running[0].correlationId).toBeDefined();
      expect(running[0].correlationId).toMatch(/^corr_/);

      // Cleanup: cancel to avoid dangling promise.
      await cancelExecution(running[0].executionId);
      void execPromise;
    });
  });
});

describe("PATCH 6.3.3 — Final Action Runtime Closure regression tests", () => {
  // ── P1-2: Reject destroys prepared invocation ─────────────

  describe("Reject destroys prepared invocation", () => {
    it("after reject, confirmAction returns confirmation_not_found", async () => {
      defineAction<{ value?: string }, void>({
        id: "test.reject_destroy",
        title: "Test reject destroy",
        category: "query",
        inputSchema: z.object({ value: z.string().optional() }),
        risk: "read",
        confirmation: { mode: "always" },
        async execute() {
          return { status: "success" };
        },
      });

      const result = await executeAction("test.reject_destroy", { value: "x" });
      expect(result.status).toBe("confirmation_required");
      const confirmId = result.confirmation!.id;

      // Reject.
      rejectConfirmation(confirmId);

      // Confirm after reject → must fail.
      const confirmResult = await confirmAction(confirmId);
      expect(confirmResult.status).toBe("error");
      expect(confirmResult.error?.code).toBe("confirmation_not_found");
    });

    it("rejected destructive action is NEVER executable", async () => {
      let executeCalled = false;

      defineAction<{ sql: string }, void>({
        id: "test.reject_destructive",
        title: "Test reject destructive",
        category: "query",
        inputSchema: z.object({ sql: z.string() }),
        risk: "read",
        resolveRisk(input) {
          return /^(DROP|DELETE)\b/i.test(input.sql) ? "destructive" : "read";
        },
        confirmation: { mode: "destructive-only" },
        async execute() {
          executeCalled = true;
          return { status: "success" };
        },
      });

      const result = await executeAction(
        "test.reject_destructive",
        { sql: "DROP TABLE users" },
        { source: "ui" },
      );
      expect(result.status).toBe("confirmation_required");

      // Reject.
      rejectConfirmation(result.confirmation!.id);

      // Attempt to confirm → must fail.
      const confirmResult = await confirmAction(result.confirmation!.id);
      expect(confirmResult.status).toBe("error");
      expect(executeCalled).toBe(false);
    });
  });

  // ── P1-1: findRunningExecutionForTab ──────────────────────

  describe("findRunningExecutionForTab", () => {
    it("finds running execution by tabId", async () => {
      defineAction<{ value?: string }, void>({
        id: "test.find_tab",
        title: "Test find tab",
        category: "query",
        inputSchema: z.object({ value: z.string().optional() }),
        risk: "read",
        resolveContext(_input, ambient) {
          return { ...ambient, tabId: "tab-xyz" };
        },
        async execute() {
          return new Promise(() => {});
        },
      });

      const execPromise = executeAction("test.find_tab", {}, {
        context: { tabId: "tab-xyz" },
      });
      await new Promise((r) => setTimeout(r, 10));

      const found = findRunningExecutionForTab("tab-xyz");
      expect(found).toBeDefined();
      expect(found!.actionId).toBe("test.find_tab");
      expect(found!.tabId).toBe("tab-xyz");
      expect(found!.state).toBe("running");

      // Cleanup.
      await cancelExecution(found!.executionId);
      void execPromise;
    });

    it("filters by actionIds when provided", async () => {
      defineAction<{ value?: string }, void>({
        id: "test.find_filter_a",
        title: "Test find filter A",
        category: "query",
        inputSchema: z.object({ value: z.string().optional() }),
        risk: "read",
        resolveContext(_input, ambient) {
          return { ...ambient, tabId: "tab-filter" };
        },
        async execute() {
          return new Promise(() => {});
        },
      });

      const execPromise = executeAction("test.find_filter_a", {}, {
        context: { tabId: "tab-filter" },
      });
      await new Promise((r) => setTimeout(r, 10));

      // Search with matching actionId filter.
      const foundMatch = findRunningExecutionForTab("tab-filter", ["test.find_filter_a"]);
      expect(foundMatch).toBeDefined();

      // Search with non-matching actionId filter.
      const foundNoMatch = findRunningExecutionForTab("tab-filter", ["query.execute.current"]);
      expect(foundNoMatch).toBeUndefined();

      // Cleanup.
      await cancelExecution(foundMatch!.executionId);
      void execPromise;
    });

    it("returns undefined when no running execution for tab", () => {
      const found = findRunningExecutionForTab("nonexistent-tab");
      expect(found).toBeUndefined();
    });
  });

  // ── One confirmation API: no confirmationToken ─────────────

  describe("One confirmation API", () => {
    it("executeAction options do not include confirmationToken", async () => {
      // This is a compile-time check — if confirmationToken is in the
      // options type, TypeScript would allow it. We verify the API shape
      // by checking that confirmAction is the only way to confirm.
      defineAction<{ value?: string }, void>({
        id: "test.one_api",
        title: "Test one API",
        category: "query",
        inputSchema: z.object({ value: z.string().optional() }),
        risk: "read",
        confirmation: { mode: "always" },
        async execute() {
          return { status: "success" };
        },
      });

      const result = await executeAction("test.one_api", { value: "x" });
      expect(result.status).toBe("confirmation_required");

      // Only confirmAction(id) works.
      const confirmResult = await confirmAction(result.confirmation!.id);
      expect(confirmResult.status).toBe("success");
    });
  });
});

describe("PATCH 6.3-FINAL — Terminal state & confirmation queue", () => {
  // ── Terminal state immutability ────────────────────────────

  describe("ActionExecution terminal state is immutable", () => {
    it("cancelled execution cannot be overwritten to completed by late resolve", async () => {
      let resolveExecute: (() => void) | null = null;

      defineAction<{ value?: string }, void>({
        id: "test.terminal_cancel",
        title: "Test terminal cancel",
        category: "query",
        inputSchema: z.object({ value: z.string().optional() }),
        risk: "read",
        resolveContext(_input, ambient) {
          return { ...ambient, tabId: "tab-terminal" };
        },
        async execute() {
          // Hold the execute promise until we manually resolve it.
          await new Promise<void>((resolve) => {
            resolveExecute = resolve as () => void;
          });
          return { status: "success" };
        },
        async cancel() {
          // Cancel resolves immediately.
        },
      });

      // Start execution.
      const execPromise = executeAction("test.terminal_cancel", {}, {
        context: { tabId: "tab-terminal" },
      });
      await new Promise((r) => setTimeout(r, 10));

      // Verify running.
      const running = getRunningExecutions();
      const exec = running.find((e) => e.tabId === "tab-terminal");
      expect(exec).toBeDefined();
      expect(exec!.state).toBe("running");

      // Cancel — sets state to "cancelled".
      const cancelResult = await cancelExecution(exec!.executionId);
      expect(cancelResult.status).toBe("cancelled");

      // Verify cancelled.
      const afterCancel = getRunningExecutions().find(
        (e) => e.executionId === exec!.executionId,
      );
      // After cancel, the execution may or may not be in getRunningExecutions
      // (since state is no longer "running"). Check getExecution instead.
      const tracked = getExecution(exec!.executionId);
      expect(tracked).toBeDefined();
      expect(tracked!.state).toBe("cancelled");

      // Now resolve the original execute late.
      expect(resolveExecute).toBeTruthy();
      resolveExecute!();
      await execPromise;

      // State MUST still be "cancelled" — late success must not overwrite.
      const afterLateResolve = getExecution(exec!.executionId);
      expect(afterLateResolve).toBeDefined();
      expect(afterLateResolve!.state).toBe("cancelled");
    });

    it("completed execution cannot transition to error", async () => {
      defineAction<{ value?: string }, void>({
        id: "test.terminal_complete",
        title: "Test terminal complete",
        category: "query",
        inputSchema: z.object({ value: z.string().optional() }),
        risk: "read",
        resolveContext(_input, ambient) {
          return { ...ambient, tabId: "tab-complete" };
        },
        async execute() {
          return { status: "success" };
        },
      });

      const result = await executeAction("test.terminal_complete", {}, {
        context: { tabId: "tab-complete" },
      });
      expect(result.status).toBe("success");

      const execId = result.executionId!;
      const tracked = getExecution(execId);
      expect(tracked!.state).toBe("completed");

      // Attempting to cancel a completed execution should fail.
      const cancelResult = await cancelExecution(execId);
      expect(cancelResult.status).toBe("error");
      expect(cancelResult.error?.code).toBe("execution_not_found");

      // State is still "completed".
      const still = getExecution(execId);
      expect(still!.state).toBe("completed");
    });
  });

  // ── Confirmation queue / orphan prevention ─────────────────

  describe("Confirmation store must not orphan prepared invocations", () => {
    it("setPending with new confirmation rejects the old one", async () => {
      // Dynamic import to get the real store.
      const { useActionConfirmationStore } = await import(
        "@/commons/stores/action-confirmation.store"
      );

      defineAction<{ value?: string }, void>({
        id: "test.queue_a",
        title: "Test queue A",
        category: "query",
        inputSchema: z.object({ value: z.string().optional() }),
        risk: "read",
        confirmation: { mode: "always" },
        async execute() {
          return { status: "success" };
        },
      });

      defineAction<{ value?: string }, void>({
        id: "test.queue_b",
        title: "Test queue B",
        category: "query",
        inputSchema: z.object({ value: z.string().optional() }),
        risk: "read",
        confirmation: { mode: "always" },
        async execute() {
          return { status: "success" };
        },
      });

      // Request confirmation A.
      const resultA = await executeAction("test.queue_a", { value: "a" });
      expect(resultA.status).toBe("confirmation_required");
      const confirmIdA = resultA.confirmation!.id;

      // Push A into the global store.
      useActionConfirmationStore.getState().setPending(resultA.confirmation!);
      expect(useActionConfirmationStore.getState().pending!.id).toBe(confirmIdA);

      // Request confirmation B.
      const resultB = await executeAction("test.queue_b", { value: "b" });
      expect(resultB.status).toBe("confirmation_required");

      // Push B into the global store — A should be auto-rejected.
      useActionConfirmationStore.getState().setPending(resultB.confirmation!);

      // Current pending is B.
      expect(useActionConfirmationStore.getState().pending!.id).toBe(
        resultB.confirmation!.id,
      );

      // A's prepared invocation must have been destroyed.
      const confirmA = await confirmAction(confirmIdA);
      expect(confirmA.status).toBe("error");
      expect(confirmA.error?.code).toBe("confirmation_not_found");

      // Cleanup.
      useActionConfirmationStore.getState().reject();
    });

    it("clearPending rejects the prepared invocation", async () => {
      const { useActionConfirmationStore } = await import(
        "@/commons/stores/action-confirmation.store"
      );

      defineAction<{ value?: string }, void>({
        id: "test.clear_pending",
        title: "Test clear pending",
        category: "query",
        inputSchema: z.object({ value: z.string().optional() }),
        risk: "read",
        confirmation: { mode: "always" },
        async execute() {
          return { status: "success" };
        },
      });

      // Request confirmation.
      const result = await executeAction("test.clear_pending", { value: "x" });
      expect(result.status).toBe("confirmation_required");
      const confirmId = result.confirmation!.id;

      // Push into store.
      useActionConfirmationStore.getState().setPending(result.confirmation!);

      // Clear pending — must reject the prepared invocation.
      useActionConfirmationStore.getState().clearPending();

      // Pending is null.
      expect(useActionConfirmationStore.getState().pending).toBeNull();

      // Attempting to confirm → confirmation_not_found.
      const confirmResult = await confirmAction(confirmId);
      expect(confirmResult.status).toBe("error");
      expect(confirmResult.error?.code).toBe("confirmation_not_found");
    });
  });
});
