/**
 * PATCH 6.3.3 — Production Action/Query Integration Tests.
 *
 * These tests use:
 *   - Production Action Bus (executeAction, confirmAction, cancelExecution)
 *   - Production query action definitions (query.execute.*, query.cancel, query.explain)
 *   - Production Query Runtime (executeQuery, cancelQuery)
 *   - Mock QueryService ONLY (the backend transport)
 *
 * No synthetic actions. No simulation. No copied logic.
 *
 * The goal is to verify the FULL production path:
 *   UI/Palette/Agent → Action Bus → Query Action → Query Runtime → Mock Service
 */

import { describe, expect, it, beforeEach, vi, afterEach } from "vitest";

// ─── Mocks ───────────────────────────────────────────────────

// Mock QueryService — the ONLY backend transport mock.
const mockExecute = vi.fn();
const mockExecuteMulti = vi.fn();
const mockCancel = vi.fn();
const mockExplain = vi.fn();

vi.mock("@/modules/query/services/query.service", () => ({
  createQueryService: () => ({
    execute: mockExecute,
    executeMulti: mockExecuteMulti,
    cancel: mockCancel,
    explain: mockExplain,
    getHistory: vi.fn().mockResolvedValue([]),
    save: vi.fn().mockResolvedValue({ id: "saved-1", name: "test", sql: "" }),
    listSaved: vi.fn().mockResolvedValue([]),
    deleteSaved: vi.fn().mockResolvedValue(undefined),
    createFolder: vi.fn().mockResolvedValue({ id: "f-1", name: "test" }),
    listFolders: vi.fn().mockResolvedValue([]),
    deleteFolder: vi.fn().mockResolvedValue(undefined),
    saveRunConfig: vi.fn().mockResolvedValue({}),
    listRunConfigs: vi.fn().mockResolvedValue([]),
    deleteRunConfig: vi.fn().mockResolvedValue(undefined),
  }),
}));

// Mock local history to avoid side effects.
vi.mock("../services/local-history", () => ({
  pushLocalHistory: vi.fn(),
}));

// ─── Store imports (real Zustand stores) ─────────────────────

import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { useQueryEditorContextStore } from "@/commons/stores/query-editor-context.store";
import { useActionConfirmationStore } from "@/commons/stores/action-confirmation.store";

// ─── Runtime imports (for stale cancel test) ─────────────────

import {
  cancelQuery as runtimeCancelQuery,
} from "@/modules/query/runtime/query-runtime";

// ─── Test constants ──────────────────────────────────────────

const TAB_A = "tab-test-a";
const CONN_ID = "conn-test-1";

// ─── Helpers ─────────────────────────────────────────────────

/** Set up a query tab in the workspace store. */
function setupQueryTab(tabId: string, sql: string, status: string = "idle") {
  useWorkspaceStore.setState({
    tabs: [
      {
        id: tabId,
        kind: "query" as const,
        connectionId: CONN_ID,
        data: {
          sql,
          status: status as "idle" | "running" | "success" | "error" | "cancelled",
          error: null,
          result: null,
          multiResults: null,
          explainPlan: null,
          sort: { column: null, direction: null },
          timing: null,
          activePanel: "results" as const,
          activeExecutionId: null,
          executionStartedAt: null,
          context: { database: null, schema: null },
        },
      },
    ],
    activeTabId: tabId,
  });
}

/** Reset workspace store to empty state. */
function resetWorkspace() {
  useWorkspaceStore.setState({
    tabs: [],
    activeTabId: null,
  });
  useQueryEditorContextStore.setState({ contexts: {} });
  useActionConfirmationStore.setState({ pending: null, isConfirming: false, lastResult: null });
}

// ─── Dynamic imports (ensures shared module instances) ───────

let bus: typeof import("@/commons/actions/bus");
let queryActions: typeof import("@/commons/actions/definitions/query.actions");

beforeEach(async () => {
  vi.clearAllMocks();
  resetWorkspace();

  // Dynamic import ensures bus and query.actions share the same
  // registry and module instances.
  bus = await import("@/commons/actions/bus");
  queryActions = await import("@/commons/actions/definitions/query.actions");

  // Clear stale active executions from previous tests.
  bus.resetActiveExecutions();

  // Suppress unused variable warnings — the import side-effect
  // registers the production actions in the shared registry.
  void queryActions;
});

afterEach(() => {
  resetWorkspace();
});

// ─── Production Integration Tests ────────────────────────────

describe("PATCH 6.3.3 — Production Action/Query Integration", () => {
  // A. Destructive UI query → confirmation before backend

  describe("A. Destructive query → confirmation before backend", () => {
    it("DROP TABLE returns confirmation_required and does NOT call service.execute", async () => {
      setupQueryTab(TAB_A, "DROP TABLE users");

      const result = await bus.executeAction(
        "query.execute.all",
        { tabId: TAB_A },
        { source: "ui" },
      );

      expect(result.status).toBe("confirmation_required");
      expect(result.confirmation).toBeDefined();
      expect(result.confirmation!.actionId).toBe("query.execute.all");
      // Backend must NOT be called.
      expect(mockExecute).not.toHaveBeenCalled();
      expect(mockExecuteMulti).not.toHaveBeenCalled();
    });
  });

  // B. Confirm executes frozen original SQL/context

  describe("B. Confirm executes frozen SQL", () => {
    it("confirm executes the ORIGINAL SQL even if tab SQL changed", async () => {
      setupQueryTab(TAB_A, "DROP TABLE users");
      mockExecuteMulti.mockResolvedValue({
        results: [],
        totalDurationMs: 10,
        error: null,
      });

      // Step 1: Request execution → confirmation_required.
      const first = await bus.executeAction(
        "query.execute.all",
        { tabId: TAB_A },
        { source: "ui" },
      );
      expect(first.status).toBe("confirmation_required");

      // Step 2: "Change" the tab SQL after confirmation was requested.
      setupQueryTab(TAB_A, "SELECT 1");

      // Step 3: Confirm — should execute the FROZEN "DROP TABLE users".
      const second = await bus.confirmAction(first.confirmation!.id);
      expect(second.status).toBe("success");

      // Backend received the FROZEN SQL, not the live "SELECT 1".
      expect(mockExecuteMulti).toHaveBeenCalledTimes(1);
      const callArgs = mockExecuteMulti.mock.calls[0];
      expect(callArgs[1]).toBe("DROP TABLE users");
    });
  });

  // C. Reject → cannot confirm later

  describe("C. Reject destroys prepared invocation", () => {
    it("after reject, confirmAction returns confirmation_not_found", async () => {
      setupQueryTab(TAB_A, "DROP TABLE users");

      const first = await bus.executeAction(
        "query.execute.all",
        { tabId: TAB_A },
        { source: "ui" },
      );
      expect(first.status).toBe("confirmation_required");
      const confirmId = first.confirmation!.id;

      // Reject.
      bus.rejectConfirmation(confirmId);

      // Attempt to confirm → must fail.
      const second = await bus.confirmAction(confirmId);
      expect(second.status).toBe("error");
      expect(second.error?.code).toBe("confirmation_not_found");

      // Backend must NOT have been called.
      expect(mockExecuteMulti).not.toHaveBeenCalled();
    });
  });

  // D. query.cancel resolves ActionExecutionId correctly

  describe("D. query.cancel resolves ActionExecutionId from bus", () => {
    it("query.cancel finds the running ActionExecution via findRunningExecutionForTab", async () => {
      setupQueryTab(TAB_A, "SELECT 1");
      mockExecute.mockImplementation(() => new Promise(() => {})); // never resolves

      // Start a production query.execute.current.
      const execPromise = bus.executeAction(
        "query.execute.current",
        { tabId: TAB_A },
        { source: "ui" },
      );
      await new Promise((r) => setTimeout(r, 20));

      // Verify a running execution exists in the bus.
      const running = bus.getRunningExecutions();
      const queryExec = running.find(
        (e) => e.actionId === "query.execute.current" && e.tabId === TAB_A,
      );
      expect(queryExec).toBeDefined();
      expect(queryExec!.externalExecutionId).toBeDefined();

      // Now call query.cancel — it must find the execution via findRunningExecutionForTab.
      const cancelResult = await bus.executeAction(
        "query.cancel",
        { tabId: TAB_A },
        { source: "ui" },
      );
      expect(cancelResult.status).toBe("success");

      // The backend cancel must have been called with the EXTERNAL (backend) execution ID.
      expect(mockCancel).toHaveBeenCalledWith(queryExec!.externalExecutionId);

      void execPromise;
    });
  });

  // E. Backend cancel receives external/backend ID

  describe("E. Backend cancel receives external ID", () => {
    it("QueryService.cancel is called with the backend execution ID, not the ActionExecutionId", async () => {
      setupQueryTab(TAB_A, "SELECT 1");
      mockExecute.mockImplementation(() => new Promise(() => {}));

      const execPromise = bus.executeAction(
        "query.execute.current",
        { tabId: TAB_A },
        { source: "ui" },
      );
      await new Promise((r) => setTimeout(r, 20));

      const running = bus.getRunningExecutions();
      const queryExec = running.find(
        (e) => e.actionId === "query.execute.current" && e.tabId === TAB_A,
      );
      expect(queryExec).toBeDefined();

      const actionExecId = queryExec!.executionId; // exec_xxx
      const backendExecId = queryExec!.externalExecutionId; // UUID

      // These must be DIFFERENT identities.
      expect(actionExecId).not.toBe(backendExecId);
      expect(actionExecId).toMatch(/^exec_/);

      // Cancel via bus.
      const cancelResult = await bus.cancelExecution(actionExecId);
      expect(cancelResult.status).toBe("cancelled");

      // Backend received the BACKEND ID, not the action ID.
      expect(mockCancel).toHaveBeenCalledWith(backendExecId);
      expect(mockCancel).not.toHaveBeenCalledWith(actionExecId);

      void execPromise;
    });
  });

  // F. Cancel failure → cancel_failed

  describe("F. Cancel failure → error", () => {
    it("backend cancel throw → cancel_failed, NOT cancelled", async () => {
      setupQueryTab(TAB_A, "SELECT 1");
      mockExecute.mockImplementation(() => new Promise(() => {}));
      mockCancel.mockRejectedValue(new Error("Backend refused"));

      const execPromise = bus.executeAction(
        "query.execute.current",
        { tabId: TAB_A },
        { source: "ui" },
      );
      await new Promise((r) => setTimeout(r, 20));

      const running = bus.getRunningExecutions();
      const queryExec = running.find(
        (e) => e.actionId === "query.execute.current" && e.tabId === TAB_A,
      );

      const cancelResult = await bus.cancelExecution(queryExec!.executionId);
      expect(cancelResult.status).toBe("error");
      expect(cancelResult.error?.code).toBe("cancel_failed");

      void execPromise;
    });
  });

  // G. Second same-tab query → query_running (unavailable)

  describe("G. Same-tab concurrency guard", () => {
    it("second execute on same tab returns action_unavailable while running", async () => {
      setupQueryTab(TAB_A, "SELECT 1");
      mockExecute.mockImplementation(() => new Promise(() => {}));

      // First execution → starts running.
      const exec1 = bus.executeAction(
        "query.execute.current",
        { tabId: TAB_A },
        { source: "ui" },
      );
      await new Promise((r) => setTimeout(r, 20));

      // The runtime sets tab status to "running".
      const tab = useWorkspaceStore.getState().tabs.find((t) => t.id === TAB_A);
      expect(tab?.kind === "query" && tab.data.status).toBe("running");

      // Second execution → should be unavailable.
      const exec2 = await bus.executeAction(
        "query.execute.current",
        { tabId: TAB_A },
        { source: "ui" },
      );
      expect(exec2.status).toBe("error");
      expect(exec2.error?.code).toBe("action_unavailable");
      expect(exec2.error?.message).toContain("query_running");

      void exec1;
    });
  });

  // H. Stale cancel does not clobber newer execution

  describe("H. Stale cancel does not clobber newer execution", () => {
    it("late cancel of exec1 does not overwrite exec2 workspace state", async () => {
      setupQueryTab(TAB_A, "SELECT 1");

      const EXEC1_ID = "backend-exec-1";
      const EXEC2_ID = "backend-exec-2";

      // Delayed backend cancel.
      mockCancel.mockImplementation(
        () => new Promise<void>((resolve) => setTimeout(resolve, 50)),
      );

      // Step 1: exec1 is active in the runtime.
      // Manually set the workspace state as if executeQuery set it.
      useWorkspaceStore.setState((s) => ({
        tabs: s.tabs.map((t) =>
          t.id === TAB_A && t.kind === "query"
            ? {
                ...t,
                data: {
                  ...t.data,
                  status: "running" as const,
                  activeExecutionId: EXEC1_ID,
                  executionStartedAt: 1000,
                },
              }
            : t,
        ),
      }));

      // Step 2: Begin cancelling exec1 (backend cancel is delayed).
      const cancelPromise = runtimeCancelQuery({
        tabId: TAB_A,
        executionId: EXEC1_ID,
      });

      // Wait for cancel to be called but not yet resolved.
      await new Promise((r) => setTimeout(r, 10));
      expect(mockCancel).toHaveBeenCalledWith(EXEC1_ID);

      // Step 3: Before cancel resolves, install exec2 state
      // (simulating a newer execution starting on the same tab).
      useWorkspaceStore.setState((s) => ({
        tabs: s.tabs.map((t) =>
          t.id === TAB_A && t.kind === "query"
            ? {
                ...t,
                data: {
                  ...t.data,
                  status: "running" as const,
                  activeExecutionId: EXEC2_ID,
                  executionStartedAt: 2000,
                },
              }
            : t,
        ),
      }));

      // Step 4: Let the cancel of exec1 resolve.
      await cancelPromise;

      // Step 5: Assert exec2 state is UNAFFECTED.
      const tab = useWorkspaceStore.getState().tabs.find((t) => t.id === TAB_A);
      expect(tab?.kind).toBe("query");
      if (tab?.kind === "query") {
        // The stale guard must have prevented exec1's cancel from
        // overwriting exec2's state.
        expect(tab.data.activeExecutionId).toBe(EXEC2_ID);
        expect(tab.data.status).toBe("running");
        expect(tab.data.executionStartedAt).toBe(2000);
      }
    });
  });

  // I. execute.all partial failure → error + results preserved

  describe("I. Partial failure → error with preserved results", () => {
    it("multi execution with partial failure returns error status", async () => {
      setupQueryTab(TAB_A, "SELECT 1; INVALID SQL;");
      mockExecuteMulti.mockResolvedValue({
        results: [
          { columns: [{ name: "id", type: "int64" }], rows: [{ id: 1 }], rowCount: 1, durationMs: 5 },
        ],
        totalDurationMs: 50,
        error: [1, "syntax error at statement 2"],
      });

      const result = await bus.executeAction(
        "query.execute.all",
        { tabId: TAB_A },
        { source: "ui" },
      );

      expect(result.status).toBe("error");
      expect(result.error?.code).toBe("partial_execution_failure");
      // Results must be preserved in data.
      expect(result.data).toBeDefined();
      expect(result.data!.completedResults).toBe(1);
      expect(result.data!.failedStatement).toBe(1);
    });
  });

  // J. Command Palette destructive query → global confirmation host

  describe("J. Command Palette destructive → global confirmation store", () => {
    it("commandFromAction().execute() routes destructive confirmation to global store", async () => {
      setupQueryTab(TAB_A, "DROP TABLE users");

      // Import the real command adapter.
      const { commandFromAction } = await import("@/commons/actions/command-adapter");

      // Create a real Command from the action definition.
      const command = commandFromAction("query.execute.all", {
        inputProvider: () => ({ tabId: TAB_A }),
      });

      // Invoke the command's execute — this is the REAL code path.
      await command.execute!();

      // Verify the global store has the pending confirmation.
      const pending = useActionConfirmationStore.getState().pending;
      expect(pending).toBeDefined();
      expect(pending!.actionId).toBe("query.execute.all");
      expect(pending!.risk).toBe("destructive");
      expect(pending!.source).toBe("command-palette");

      // Confirm through the global store.
      mockExecuteMulti.mockResolvedValue({
        results: [],
        totalDurationMs: 10,
        error: null,
      });
      const confirmResult = await useActionConfirmationStore.getState().confirm();
      expect(confirmResult.status).toBe("success");

      // Store should be cleared after confirm.
      expect(useActionConfirmationStore.getState().pending).toBeNull();
    });
  });

  // K. Explain UI → query.explain Action

  describe("K. Explain goes through Action Platform", () => {
    it("query.explain action calls runtime.explainQuery and returns plan", async () => {
      setupQueryTab(TAB_A, "SELECT * FROM users");
      const mockPlan = {
        plan: "Seq Scan on users  (cost=0.00..35.50 rows=2550 width=36)",
        children: [],
      };
      mockExplain.mockResolvedValue(mockPlan);

      const result = await bus.executeAction(
        "query.explain",
        { tabId: TAB_A },
        { source: "ui" },
      );

      expect(result.status).toBe("success");
      expect(result.data).toBeDefined();
      expect((result.data as { plan: typeof mockPlan }).plan).toEqual(mockPlan);

      // Backend explain was called with correct args.
      expect(mockExplain).toHaveBeenCalledWith(CONN_ID, "SELECT * FROM users");
    });
  });

  // ─── Additional production invariants ──────────────────────

  describe("Cancel identity: ActionExecutionId ≠ BackendExecutionId", () => {
    it("tab.data.activeExecutionId stores backend ID, NOT ActionExecutionId", async () => {
      setupQueryTab(TAB_A, "SELECT 1");
      mockExecute.mockImplementation(() => new Promise(() => {}));

      const execPromise = bus.executeAction(
        "query.execute.current",
        { tabId: TAB_A },
        { source: "ui" },
      );
      await new Promise((r) => setTimeout(r, 20));

      // The runtime stores the backend execution ID in tab.data.activeExecutionId.
      const tab = useWorkspaceStore.getState().tabs.find((t) => t.id === TAB_A);
      const tabActiveExecId = tab?.kind === "query" ? tab.data.activeExecutionId : null;

      // The bus has its own ActionExecutionId.
      const running = bus.getRunningExecutions();
      const actionExec = running.find(
        (e) => e.actionId === "query.execute.current" && e.tabId === TAB_A,
      );

      // These MUST be different.
      expect(tabActiveExecId).toBe(actionExec!.externalExecutionId);
      expect(tabActiveExecId).not.toBe(actionExec!.executionId);

      // findRunningExecutionForTab finds by tabId, NOT by activeExecutionId.
      const found = bus.findRunningExecutionForTab(TAB_A, ["query.execute.current"]);
      expect(found).toBeDefined();
      expect(found!.executionId).toBe(actionExec!.executionId);

      void execPromise;
    });
  });

  describe("Reject destroys prepared: no stale execution possible", () => {
    it("reject then confirm → confirmation_not_found, backend NOT called", async () => {
      setupQueryTab(TAB_A, "DELETE FROM users WHERE 1=1");

      const result = await bus.executeAction(
        "query.execute.all",
        { tabId: TAB_A },
        { source: "ui" },
      );
      expect(result.status).toBe("confirmation_required");
      const confirmId = result.confirmation!.id;

      // User rejects.
      bus.rejectConfirmation(confirmId);

      // Attempting to confirm after reject must fail.
      const confirmResult = await bus.confirmAction(confirmId);
      expect(confirmResult.status).toBe("error");
      expect(confirmResult.error?.code).toBe("confirmation_not_found");

      // Backend must NOT have been called at any point.
      expect(mockExecuteMulti).not.toHaveBeenCalled();
    });
  });
});
