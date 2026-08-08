import { getAction } from "./registry";
import { buildActionContext } from "./context";
import { emitAuditEvent } from "./audit";

import type {
  ActionConfirmation,
  ActionDefinition,
  ActionExecutionContext,
  ActionExecution,
  ActionId,
  ActionProgress,
  ActionResult,
  ActionRisk,
  ActionSource,
} from "./types";

/**
 * Central execution engine for all actions.
 *
 * Every action invocation flows through this bus, which handles:
 *   1. Input validation (via the action's inputSchema)
 *   2. Availability check (enabled/disabled)
 *   3. Safety gate (risk + confirmation protocol)
 *   4. Execution delegation to the action handler
 *   5. Audit event emission (start / complete / error / cancel)
 *   6. Execution tracking (for long-running actions & progress)
 *
 * The bus is UI-framework agnostic — it can be called from React
 * components, keyboard handlers, the command palette, or MCP.
 */

// ─── Pending confirmations ───────────────────────────────────

const pendingConfirmations = new Map<string, ActionConfirmation>();

// ─── Active executions (long-running tracking) ───────────────

const activeExecutions = new Map<string, ActionExecution>();

// ─── Execute ─────────────────────────────────────────────────

/**
 * Execute an action by its stable ID.
 *
 * @param actionId  Stable action identifier (e.g. "query.execute.current")
 * @param input     Input matching the action's inputSchema
 * @param options   Source, context overrides, confirmation token
 */
export async function executeAction<TOutput = unknown>(
  actionId: ActionId,
  input?: Record<string, unknown>,
  options?: {
    source?: ActionSource;
    context?: Partial<ActionExecutionContext>;
    confirmationToken?: string;
  },
): Promise<ActionResult<TOutput>> {
  const definition = getAction(actionId);
  if (!definition) {
    return {
      status: "error",
      error: {
        code: "action_not_found",
        message: `Action "${actionId}" is not registered`,
      },
    };
  }

  const source = options?.source ?? "ui";
  const ctx = buildActionContext(source, options?.context);

  // ── 1. Validate input ──────────────────────────────────────
  const parseResult = definition.inputSchema.safeParse(input ?? {});
  if (!parseResult.success) {
    return {
      status: "error",
      error: {
        code: "invalid_input",
        message: `Invalid input for "${actionId}"`,
        details: parseResult.error,
      },
    };
  }

  // Zod guarantees data is present on success, but our ActionSchema
  // interface doesn't carry that narrowing — assert it here.
  const validatedInput = parseResult.data as NonNullable<typeof parseResult.data>;

  // ── 2. Availability check ──────────────────────────────────
  if (definition.availability) {
    const avail = definition.availability(ctx);
    if (avail.status === "unavailable") {
      return {
        status: "error",
        error: {
          code: "action_unavailable",
          message: avail.reason ?? `Action "${actionId}" is not available`,
        },
      };
    }
  }

  // ── 2b. Dynamic risk resolution ─────────────────────────────
  let effectiveRisk = definition.risk ?? "read";
  if (definition.resolveRisk) {
    effectiveRisk = definition.resolveRisk(validatedInput, ctx);
  }

  // ── 3. Confirmation gate ───────────────────────────────────
  if (requiresConfirmation(definition, source, effectiveRisk)) {
    // If caller already has a valid confirmation token, proceed.
    if (options?.confirmationToken) {
      const conf = pendingConfirmations.get(options.confirmationToken);
      if (conf && conf.actionId === actionId) {
        pendingConfirmations.delete(options.confirmationToken);
        // Fall through to execution.
      } else {
        // Token doesn't match this action — reject.
        return {
          status: "error",
          error: {
            code: "confirmation_mismatch",
            message: `Confirmation token does not match action "${actionId}"`,
          },
        } as ActionResult<TOutput>;
      }
    } else {
      return createConfirmationResponse(
        definition,
        ctx,
        input ?? {},
        options?.context,
        source,
      ) as ActionResult<TOutput>;
    }
  }

  // ── 4. Execute ─────────────────────────────────────────────
  const executionId = `exec_${crypto.randomUUID()}`;
  const startedAt = new Date().toISOString();

  activeExecutions.set(executionId, {
    executionId,
    actionId,
    state: "running",
    startedAt: Date.now(),
  });

  emitAuditEvent({
    actionId,
    source,
    startedAt,
    status: "started",
    connectionId: ctx.connectionId,
    correlationId: ctx.correlationId,
    executionId,
  });

  try {
    const result = await definition.execute(
      validatedInput,
      ctx,
    );

    const execution = activeExecutions.get(executionId);
    if (execution) {
      execution.state =
        result.status === "cancelled" ? "cancelled" : "completed";
      execution.result = result;
    }

    emitAuditEvent({
      actionId,
      source,
      startedAt,
      completedAt: new Date().toISOString(),
      status:
        result.status === "error"
          ? "error"
          : result.status === "cancelled"
            ? "cancelled"
            : "completed",
      connectionId: ctx.connectionId,
      correlationId: ctx.correlationId,
      executionId,
      durationMs: Date.now() - new Date(startedAt).getTime(),
    });

    return { ...result, executionId: result.executionId ?? executionId } as ActionResult<TOutput>;
  } catch (err) {
    const message =
      err instanceof Error ? err.message : "Unknown action error";

    const execution = activeExecutions.get(executionId);
    if (execution) {
      execution.state = "error";
      execution.result = {
        status: "error",
        error: { code: "execution_error", message },
      };
    }

    emitAuditEvent({
      actionId,
      source,
      startedAt,
      completedAt: new Date().toISOString(),
      status: "error",
      connectionId: ctx.connectionId,
      correlationId: ctx.correlationId,
      executionId,
      durationMs: Date.now() - new Date(startedAt).getTime(),
    });

    return {
      status: "error",
      error: { code: "execution_error", message },
      executionId,
    };
  }
}

// ─── Confirmation helpers ────────────────────────────────────

function requiresConfirmation(
  def: ActionDefinition,
  _source: ActionSource,
  risk?: ActionRisk,
): boolean {
  if (!def.confirmation) return false;
  const effectiveRisk = risk ?? def.risk ?? "read";
  if (def.confirmation.mode === "always") return true;
  if (def.confirmation.mode === "destructive-only") {
    // Even agent/MCP sources must confirm destructive actions.
    return effectiveRisk === "destructive";
  }
  return false;
}

function createConfirmationResponse(
  def: ActionDefinition,
  ctx: ActionExecutionContext,
  input: Record<string, unknown>,
  contextOverrides?: Partial<ActionExecutionContext>,
  source?: ActionSource,
): ActionResult {
  const confirmationId = `confirm_${crypto.randomUUID()}`;
  const confirmation: ActionConfirmation = {
    id: confirmationId,
    actionId: def.id,
    message:
      def.confirmation?.messageKey ??
      `Confirm ${def.title}?`,
    risk: def.risk ?? "write",
    input,
    context: contextOverrides,
    source,
    createdAt: Date.now(),
  };

  pendingConfirmations.set(confirmationId, confirmation);

  return {
    status: "confirmation_required",
    confirmation,
    effects: [
      {
        type: "action.confirmation.requested",
        confirmationId,
        actionId: def.id,
        source: ctx.source,
      },
    ],
  };
}

/**
 * Confirm a pending action and re-execute it.
 *
 * Called by the UI after the user approves a confirmation dialog,
 * or by MCP after the end-user confirms remotely.
 */
export async function confirmAction(
  confirmationId: string,
): Promise<ActionResult> {
  const confirmation = pendingConfirmations.get(confirmationId);
  if (!confirmation) {
    return {
      status: "error",
      error: {
        code: "confirmation_not_found",
        message: `Confirmation "${confirmationId}" not found or already consumed`,
      },
    };
  }

  // Re-execute with the original input, context, and confirmation token.
  return executeAction(confirmation.actionId, confirmation.input, {
    source: confirmation.source ?? "ui",
    context: confirmation.context,
    confirmationToken: confirmationId,
  });
}

/** List all pending confirmations (for UI rendering). */
export function getPendingConfirmations(): readonly ActionConfirmation[] {
  return Array.from(pendingConfirmations.values());
}

/** Discard a pending confirmation (user rejected). */
export function rejectConfirmation(confirmationId: string): void {
  pendingConfirmations.delete(confirmationId);
}

// ─── Availability query ──────────────────────────────────────

/**
 * Check whether an action is currently available.
 *
 * Used by both UI (button disabled state) and MCP (tool availability).
 */
export function isActionAvailable(
  actionId: ActionId,
  contextOverrides?: Partial<ActionExecutionContext>,
): { available: boolean; reason?: string } {
  const def = getAction(actionId);
  if (!def) return { available: false, reason: "action_not_found" };

  if (!def.availability) return { available: true };

  const ctx = buildActionContext("ui", contextOverrides);
  const avail = def.availability(ctx);
  if (avail.status === "unavailable") {
    return { available: false, reason: avail.reason };
  }
  return { available: true };
}

// ─── Execution tracking ──────────────────────────────────────

/** Get the status of a running or completed execution. */
export function getExecution(
  executionId: string,
): ActionExecution | undefined {
  return activeExecutions.get(executionId);
}

/** Update progress for a running execution. */
export function updateExecutionProgress(
  executionId: string,
  progress: ActionProgress,
): void {
  const exec = activeExecutions.get(executionId);
  if (exec) {
    exec.progress = progress;
  }
}

/** Cancel a running execution. */
export async function cancelExecution(
  executionId: string,
): Promise<ActionResult> {
  const exec = activeExecutions.get(executionId);
  if (!exec || exec.state !== "running") {
    return {
      status: "error",
      error: {
        code: "execution_not_found",
        message: `Execution "${executionId}" is not running`,
      },
    };
  }

  // Delegate to the action's cancel mechanism if available.
  const def = getAction(exec.actionId);
  if (def?.cancel) {
    try {
      const ctx = buildActionContext("ui");
      await def.cancel(exec, ctx);
    } catch {
      // Cancel hook failed — still mark as cancelled but note the error.
    }
  }

  exec.state = "cancelled";
  return { status: "cancelled", executionId };
}

/** Remove completed executions from the tracking map. */
export function cleanupExecutions(maxAgeMs = 60_000): void {
  const now = Date.now();
  for (const [id, exec] of activeExecutions) {
    if (
      exec.state !== "running" &&
      now - exec.startedAt > maxAgeMs
    ) {
      activeExecutions.delete(id);
    }
  }
}
