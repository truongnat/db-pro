import { useConnectionStore } from "@/commons/stores/connection.store";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";

import type { ActionExecutionContext, ActionSource } from "./types";

let correlationCounter = 0;

/** Generate a unique correlation ID for tracing action flows. */
export function generateCorrelationId(): string {
  correlationCounter += 1;
  return `corr_${Date.now()}_${correlationCounter}`;
}

/**
 * Build an ActionExecutionContext from current application state.
 *
 * This is the bridge between the "ambient" state (active tab, active
 * connection, etc.) and the explicit parameters an action receives.
 *
 * Resolution order:
 *   1. Explicit overrides (caller-provided values)
 *   2. Target tab (if tabId override points to a specific tab)
 *   3. Active tab (ambient workspace state)
 *   4. Explorer connection (fallback for connectionId)
 */
export function buildActionContext(
  source: ActionSource,
  overrides?: Partial<ActionExecutionContext>,
): ActionExecutionContext {
  const { tabs, activeTabId } = useWorkspaceStore.getState();

  // Resolve the "target tab": explicit tabId override first, then active.
  const targetTabId = overrides?.tabId ?? activeTabId;
  const targetTab = targetTabId ? tabs.find((t) => t.id === targetTabId) : undefined;

  const explorerConnectionId = useConnectionStore.getState().explorerConnectionId;

  const connectionId =
    overrides?.connectionId ?? targetTab?.connectionId ?? explorerConnectionId ?? undefined;

  const tabId = targetTabId ?? undefined;

  let database: string | null | undefined;
  let schema: string | null | undefined;

  if (targetTab && targetTab.kind === "query") {
    database = targetTab.data.context.database;
    schema = targetTab.data.context.schema;
  }

  return {
    source,
    correlationId: overrides?.correlationId ?? generateCorrelationId(),
    workspaceId: overrides?.workspaceId,
    tabId,
    connectionId,
    database: overrides?.database ?? database,
    schema: overrides?.schema ?? schema,
    idempotencyKey: overrides?.idempotencyKey,
  };
}
