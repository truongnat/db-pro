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
      // The result should have properties — not an empty shell.
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

      // Must not be the fallback empty object.
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
      // Confirmation must carry the original input.
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

      // First call → confirmation_required.
      const first = await executeAction(
        "test.confirm_replay",
        { value: "original" },
        { source: "ui" },
      );
      expect(first.status).toBe("confirmation_required");

      // Confirm → should replay with original input.
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

      // Get a confirmation token for action A.
      const resultA = await executeAction("test.action_a", { value: "a" });
      const token = resultA.confirmation!.id;

      // Try to use it for action B → should fail.
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

      // SELECT → read → no confirmation needed.
      const readResult = await executeAction(
        "test.dynamic_risk",
        { sql: "SELECT 1" },
        { source: "agent" },
      );
      expect(readResult.status).toBe("success");

      // DELETE → destructive → confirmation required even for agent.
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
