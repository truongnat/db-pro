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
 *   2. Build ambient context
 *   3. Resolve target context from validated input (explicit targets beat ambient)
 *   4. Availability check (enabled/disabled)
 *   5. Dynamic risk resolution
 *   6. Safety gate (confirmation protocol with resolved context snapshot)
 *   7. Execution delegation to the action handler
 *   8. Audit event emission (start / complete / error / cancel)
 *   9. Execution tracking (for long-running actions & progress)
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

  // ── 2. Build ambient context ───────────────────────────────
  const ambientCtx = buildActionContext(source, options?.context);

  // ── 3. Resolve target context from validated input ─────────
  // Explicit action targets (e.g. tabId) control the COMPLETE execution context.
  let ctx = ambientCtx;
  if (definition.resolveContext) {
    const resolved = definition.resolveContext(validatedInput, ambientCtx);
    ctx = { ...ambientCtx, ...resolved };
  }

  // ── 4. Availability check ──────────────────────────────────
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

  // ── 5. Resolve executable payload ONCE ─────────────────────
  // This single resolved payload is used for:
  //   - Risk classification (resolveRisk reads from ctx.resolvedPayload)
  //   - Confirmation snapshot (frozen in pendingConfirmations)
  //   - Execution (action reads from ctx.resolvedPayload)
  // Never resolve the executable payload twice.
  let resolvedPayload: Record<string, unknown> | null = null;
  if (definition.resolvePayload) {
    resolvedPayload = definition.resolvePayload(validatedInput, ctx);
  }
  if (resolvedPayload) {
    ctx = { ...ctx, resolvedPayload };
  }

  // ── 6. Dynamic risk resolution ─────────────────────────────
  let effectiveRisk = definition.risk ?? "read";
  if (definition.resolveRisk) {
    effectiveRisk = definition.resolveRisk(validatedInput, ctx);
  }

  // ── 7. Confirmation gate ───────────────────────────────────
  if (requiresConfirmation(definition, source, effectiveRisk)) {
    // If caller already has a valid confirmation token, proceed.
    if (options?.confirmationToken) {
      const conf = pendingConfirmations.get(options.confirmationToken);
      if (conf && conf.actionId === actionId) {
        pendingConfirmations.delete(options.confirmationToken);
        // Restore the fully resolved context from the confirmation snapshot.
        if (conf.resolvedContext) {
          ctx = conf.resolvedContext;
        }
        // Restore the frozen payload on context — do NOT re-resolve from live state.
        // The action reads ctx.resolvedPayload directly; no input spreading.
        if (conf.resolvedPayload) {
          ctx = { ...ctx, resolvedPayload: conf.resolvedPayload };
        }
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
        source,
        effectiveRisk,
        resolvedPayload,
      ) as ActionResult<TOutput>;
    }
  }

  // ── 8. Execute ─────────────────────────────────────────────
  const executionId = `exec_${crypto.randomUUID()}`;
  const startedAt = new Date().toISOString();

  const execution: ActionExecution = {
    executionId,
    actionId,
    state: "running",
    startedAt: Date.now(),
  };
  activeExecutions.set(executionId, execution);

  // Inject actionExecutionId into context AFTER creating the execution.
  // Query actions use ctx.actionExecutionId for bindExternalExecutionId().
  // correlationId is for tracing ONLY — never use it for execution lookup.
  ctx = { ...ctx, actionExecutionId: executionId };

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

    const tracked = activeExecutions.get(executionId);
    if (tracked) {
      switch (result.status) {
        case "success":
          tracked.state = "completed";
          break;
        case "error":
          tracked.state = "error";
          break;
        case "cancelled":
          tracked.state = "cancelled";
          break;
        default:
          tracked.state = "completed";
      }
      tracked.result = result;
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

    // The bus owns the executionId. Action handlers must NOT override it.
    // This ensures callers always get the ActionExecutionId, not the
    // backend execution ID. Backend identity is in externalExecutionId.
    return { ...result, executionId } as ActionResult<TOutput>;
  } catch (err) {
    const message =
      err instanceof Error ? err.message : "Unknown action error";

    const tracked = activeExecutions.get(executionId);
    if (tracked) {
      tracked.state = "error";
      tracked.result = {
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
  resolvedCtx: ActionExecutionContext,
  input: Record<string, unknown>,
  source?: ActionSource,
  effectiveRisk?: ActionRisk,
  resolvedPayload?: Record<string, unknown> | null,
): ActionResult {
  const confirmationId = `confirm_${crypto.randomUUID()}`;
  const confirmation: ActionConfirmation = {
    id: confirmationId,
    actionId: def.id,
    message:
      def.confirmation?.messageKey ??
      `Confirm ${def.title}?`,
    risk: effectiveRisk ?? def.risk ?? "write",
    input,
    // Snapshot the FULLY RESOLVED context — not just overrides.
    // This ensures that switching tabs/connections before confirming
    // does NOT change the execution target.
    resolvedContext: resolvedCtx,
    // Snapshot the frozen executable payload.
    // For query actions, this is the ResolvedQueryExecution.
    // On confirm, the action uses this directly — no re-resolution.
    resolvedPayload: resolvedPayload ?? undefined,
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
        source: resolvedCtx.source,
      },
    ],
  };
}

/**
 * Confirm a pending action and re-execute it.
 *
 * Called by the UI after the user approves a confirmation dialog,
 * or by MCP after the end-user confirms remotely.
 *
 * The confirmation snapshot contains the fully resolved context,
 * so re-execution uses the ORIGINAL target — not whatever is
 * currently active.
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

  // Re-execute with the original input and the SNAPSHOT context.
  // The snapshot ensures the target doesn't drift.
  return executeAction(confirmation.actionId, confirmation.input, {
    source: confirmation.source ?? "ui",
    context: confirmation.resolvedContext
      ? {
          tabId: confirmation.resolvedContext.tabId,
          connectionId: confirmation.resolvedContext.connectionId,
          database: confirmation.resolvedContext.database,
          schema: confirmation.resolvedContext.schema,
          correlationId: confirmation.resolvedContext.correlationId,
        }
      : undefined,
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

/**
 * Bind an external (backend) execution ID to an active action execution.
 *
 * Called by action handlers right before they invoke the backend service.
 * This allows cancelExecution() to use the real backend ID for cancellation.
 */
export function bindExternalExecutionId(
  actionExecutionId: string,
  externalExecutionId: string,
): void {
  const exec = activeExecutions.get(actionExecutionId);
  if (exec) {
    exec.externalExecutionId = externalExecutionId;
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
    } catch (err) {
      // Cancel hook FAILED — do NOT mark as cancelled.
      // Report the error so the caller knows cancellation didn't succeed.
      const message = err instanceof Error ? err.message : "Cancel failed";
      exec.state = "error";
      exec.result = {
        status: "error",
        error: { code: "cancel_failed", message },
      };
      return {
        status: "error",
        error: { code: "cancel_failed", message },
        executionId,
      };
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

/**
 * Get all currently running executions.
 *
 * Exposed for testing — real cancellation tests need to observe
 * running ActionExecution IDs to verify bind + cancel flow.
 */
export function getRunningExecutions(): readonly ActionExecution[] {
  return Array.from(activeExecutions.values()).filter(
    (e) => e.state === "running",
  );
}
