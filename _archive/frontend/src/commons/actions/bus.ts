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
 *   6. Safety gate (confirmation protocol with prepared invocation snapshot)
 *   7. Execution delegation to the action handler
 *   8. Audit event emission (start / complete / error / cancel)
 *   9. Execution tracking (for long-running actions & progress)
 *
 * Confirmation API:
 *   executeAction() → confirmation_required
 *   confirmAction(id) → executes frozen prepared invocation
 *   rejectConfirmation(id) → destroys both pending + prepared maps
 *
 * The bus is UI-framework agnostic — it can be called from React
 * components, keyboard handlers, the command palette, or MCP.
 */

// ─── Pending confirmations ───────────────────────────────────

const pendingConfirmations = new Map<string, ActionConfirmation>();

// ─── Prepared invocations (frozen at confirmation time) ──────

/**
 * A prepared invocation captures the COMPLETE resolved state at the
 * moment confirmation was requested. On confirm, the bus executes
 * THIS prepared invocation directly — no live re-resolution.
 */
interface PreparedActionInvocation {
  actionId: ActionId;
  validatedInput: Record<string, unknown>;
  resolvedContext: ActionExecutionContext;
  resolvedPayload: Record<string, unknown> | null;
  effectiveRisk: ActionRisk;
  source: ActionSource;
}

const preparedInvocations = new Map<string, PreparedActionInvocation>();

// ─── Active executions (long-running tracking) ───────────────

const activeExecutions = new Map<string, ActionExecution>();

// ─── Execute ─────────────────────────────────────────────────

/**
 * Execute an action by its stable ID.
 *
 * @param actionId  Stable action identifier (e.g. "query.execute.current")
 * @param input     Input matching the action's inputSchema
 * @param options   Source and context overrides
 *
 * Confirmation is a two-step protocol:
 *   1. executeAction() returns confirmation_required with an ActionConfirmation
 *   2. confirmAction(confirmationId) executes the frozen prepared invocation
 *
 * There is NO confirmationToken fast-path. All confirmations go through
 * confirmAction(id) — the single official confirmation API.
 */
export async function executeAction<TOutput = unknown>(
  actionId: ActionId,
  input?: Record<string, unknown>,
  options?: {
    source?: ActionSource;
    context?: Partial<ActionExecutionContext>;
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

  const validatedInput = parseResult.data as NonNullable<typeof parseResult.data>;

  // ── 2. Build ambient context ───────────────────────────────
  const ambientCtx = buildActionContext(source, options?.context);

  // ── 3. Resolve target context from validated input ─────────
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
    // Store the prepared invocation for later execution on confirm.
    const confirmationId = `confirm_${crypto.randomUUID()}`;
    const confirmation: ActionConfirmation = {
      id: confirmationId,
      actionId: definition.id,
      message: definition.confirmation?.messageKey ?? `Confirm ${definition.title}?`,
      risk: effectiveRisk ?? definition.risk ?? "write",
      input: input ?? {},
      resolvedContext: ctx,
      resolvedPayload: resolvedPayload ?? undefined,
      source,
      createdAt: Date.now(),
    };

    pendingConfirmations.set(confirmationId, confirmation);
    preparedInvocations.set(confirmationId, {
      actionId: definition.id,
      validatedInput: validatedInput as unknown as Record<string, unknown>,
      resolvedContext: ctx,
      resolvedPayload,
      effectiveRisk,
      source,
    });

    return {
      status: "confirmation_required",
      confirmation,
      effects: [
        {
          type: "action.confirmation.requested",
          confirmationId,
          actionId: definition.id,
          source: ctx.source,
        },
      ],
    } as ActionResult<TOutput>;
  }

  // ── 8. Execute ─────────────────────────────────────────────
  return runExecution(
    definition,
    validatedInput as unknown as Record<string, unknown>,
    ctx,
    resolvedPayload,
    source,
  ) as Promise<ActionResult<TOutput>>;
}

// ─── Execution state machine ─────────────────────────────────

/**
 * Transition an ActionExecution to a new state.
 *
 * Terminal states (completed, error, cancelled) are IMMUTABLE.
 * Once an execution reaches a terminal state, no further transitions
 * are allowed. This prevents late async results from overwriting
 * a cancellation or error state.
 *
 * Returns true if the transition was applied, false if forbidden.
 */
function transitionExecution(
  execution: ActionExecution,
  nextState: ActionExecution["state"],
): boolean {
  // Terminal states are immutable.
  if (
    execution.state === "completed" ||
    execution.state === "error" ||
    execution.state === "cancelled"
  ) {
    return false;
  }
  // Only allow running → terminal transitions.
  if (execution.state !== "running") {
    return false;
  }
  execution.state = nextState;
  return true;
}

// ─── Execution helper ────────────────────────────────────────

/**
 * Execute a prepared/frozen action invocation.
 *
 * Shared between the normal flow and the confirmation fast-path.
 * Creates the ActionExecution, injects actionExecutionId, runs the
 * handler, tracks state, emits audit events.
 */
async function runExecution(
  definition: ActionDefinition,
  validatedInput: Record<string, unknown>,
  ctx: ActionExecutionContext,
  resolvedPayload: Record<string, unknown> | null,
  source: ActionSource,
): Promise<ActionResult> {
  const executionId = `exec_${crypto.randomUUID()}`;
  const startedAt = new Date().toISOString();

  // Ensure the frozen payload is on context (for confirmation fast-path
  // the ctx may not have it yet).
  if (resolvedPayload && !ctx.resolvedPayload) {
    ctx = { ...ctx, resolvedPayload };
  }

  const execution: ActionExecution = {
    executionId,
    actionId: definition.id,
    state: "running",
    startedAt: Date.now(),
    // Store execution context for cancellation.
    source,
    tabId: ctx.tabId,
    connectionId: ctx.connectionId,
    correlationId: ctx.correlationId,
  };
  activeExecutions.set(executionId, execution);

  // Inject actionExecutionId into context.
  ctx = { ...ctx, actionExecutionId: executionId };

  emitAuditEvent({
    actionId: definition.id,
    source,
    startedAt,
    status: "started",
    connectionId: ctx.connectionId,
    correlationId: ctx.correlationId,
    executionId,
  });

  try {
    const result = await definition.execute(validatedInput as unknown as never, ctx);

    const tracked = activeExecutions.get(executionId);
    let auditStatus: "completed" | "error" | "cancelled" = "completed";
    if (tracked) {
      // Determine the result status.
      if (result.status === "error") auditStatus = "error";
      else if (result.status === "cancelled") auditStatus = "cancelled";

      // Apply state transition — terminal states are immutable.
      // If cancelExecution() already set state to "cancelled", a late
      // success/error result MUST NOT overwrite it.
      const transitioned = transitionExecution(tracked, auditStatus);
      if (transitioned) {
        tracked.result = result;
      }
      // If not transitioned, the execution was already terminal.
      // Keep the existing state and result.
    }

    // Only emit completion audit if the state was not already terminal.
    // A late result resolving after cancellation should not produce a
    // misleading "completed" audit event.
    if (tracked && tracked.state === auditStatus) {
      emitAuditEvent({
        actionId: definition.id,
        source,
        startedAt,
        completedAt: new Date().toISOString(),
        status: auditStatus,
        connectionId: ctx.connectionId,
        correlationId: ctx.correlationId,
        executionId,
        durationMs: Date.now() - new Date(startedAt).getTime(),
      });
    }

    // The bus owns the executionId. Action handlers must NOT override it.
    return { ...result, executionId };
  } catch (err) {
    const message = err instanceof Error ? err.message : "Unknown action error";

    const tracked = activeExecutions.get(executionId);
    // Apply error state — terminal states are immutable.
    const transitioned = tracked ? transitionExecution(tracked, "error") : false;
    if (tracked && transitioned) {
      tracked.result = {
        status: "error",
        error: { code: "execution_error", message },
      };
    }

    // Only emit error audit if the transition was applied.
    if (tracked && transitioned) {
      emitAuditEvent({
        actionId: definition.id,
        source,
        startedAt,
        completedAt: new Date().toISOString(),
        status: "error",
        connectionId: ctx.connectionId,
        correlationId: ctx.correlationId,
        executionId,
        durationMs: Date.now() - new Date(startedAt).getTime(),
      });
    }

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
    return effectiveRisk === "destructive";
  }
  return false;
}

/**
 * Confirm a pending action and execute the frozen prepared invocation.
 *
 * The prepared invocation was captured at the moment confirmation was
 * requested. It contains the frozen validated input, resolved context,
 * and resolved payload. We execute THAT invocation directly — no live
 * re-resolution of editor, tab, connection, or risk.
 */
export async function confirmAction(confirmationId: string): Promise<ActionResult> {
  const prepared = preparedInvocations.get(confirmationId);
  if (!prepared) {
    return {
      status: "error",
      error: {
        code: "confirmation_not_found",
        message: `Confirmation "${confirmationId}" not found or already consumed`,
      },
    };
  }

  const definition = getAction(prepared.actionId);
  if (!definition) {
    preparedInvocations.delete(confirmationId);
    pendingConfirmations.delete(confirmationId);
    return {
      status: "error",
      error: {
        code: "action_not_found",
        message: `Action "${prepared.actionId}" is no longer registered`,
      },
    };
  }

  // Execute the frozen prepared invocation directly.
  // No live re-resolution of context, payload, or risk.
  preparedInvocations.delete(confirmationId);
  pendingConfirmations.delete(confirmationId);

  return runExecution(
    definition,
    prepared.validatedInput,
    prepared.resolvedContext,
    prepared.resolvedPayload,
    prepared.source,
  );
}

/** List all pending confirmations (for UI rendering). */
export function getPendingConfirmations(): readonly ActionConfirmation[] {
  return Array.from(pendingConfirmations.values());
}

/** Discard a pending confirmation (user rejected). Destroys BOTH maps. */
export function rejectConfirmation(confirmationId: string): void {
  pendingConfirmations.delete(confirmationId);
  preparedInvocations.delete(confirmationId);
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
export function getExecution(executionId: string): ActionExecution | undefined {
  return activeExecutions.get(executionId);
}

/** Update progress for a running execution. */
export function updateExecutionProgress(executionId: string, progress: ActionProgress): void {
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
export async function cancelExecution(executionId: string): Promise<ActionResult> {
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
  if (!def?.cancel) {
    // No cancel handler — do NOT fake cancellation.
    // The backend query keeps running; report error to caller.
    return {
      status: "error",
      error: {
        code: "cancel_not_supported",
        message: `Action "${exec.actionId}" does not support cancellation`,
      },
      executionId,
    };
  }

  try {
    // Use the STORED execution context from the ActionExecution,
    // not the CURRENT active UI tab. Cancellation belongs to the
    // execution being cancelled.
    const cancelCtx = buildActionContext(exec.source ?? "ui", {
      tabId: exec.tabId,
      connectionId: exec.connectionId,
      correlationId: exec.correlationId,
    });
    await def.cancel(exec, cancelCtx);
  } catch (err) {
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

  exec.state = "cancelled";
  return { status: "cancelled", executionId };
}

/** Remove completed executions from the tracking map. */
export function cleanupExecutions(maxAgeMs = 60_000): void {
  const now = Date.now();
  for (const [id, exec] of activeExecutions) {
    if (exec.state !== "running" && now - exec.startedAt > maxAgeMs) {
      activeExecutions.delete(id);
    }
  }
}

/** Clear all active executions — test helper only. */
export function resetActiveExecutions(): void {
  activeExecutions.clear();
}

/**
 * Get all currently running executions.
 *
 * Used by concurrency guards and the cancel identity resolver.
 */
export function getRunningExecutions(): readonly ActionExecution[] {
  return Array.from(activeExecutions.values()).filter((e) => e.state === "running");
}

/**
 * Find the running ActionExecution for a specific tab.
 *
 * This is the canonical way to resolve the ActionExecutionId from a tabId.
 * Used by query.cancel to find the correct ActionExecution to cancel —
 * never use tab.data.activeExecutionId for this, as it stores the
 * BackendExecutionId, not the ActionExecutionId.
 *
 * @param tabId      The tab to find the execution for
 * @param actionIds  Optional filter: only match these action IDs
 */
export function findRunningExecutionForTab(
  tabId: string,
  actionIds?: readonly ActionId[],
): ActionExecution | undefined {
  return Array.from(activeExecutions.values()).find(
    (e) =>
      e.state === "running" && e.tabId === tabId && (!actionIds || actionIds.includes(e.actionId)),
  );
}
