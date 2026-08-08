import { useConnectionStore } from "@/commons/stores/connection.store";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";

import type {
  ActionExecutionContext,
  ActionSource,
} from "./types";

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
 * Callers can override any field via the `overrides` parameter.
 */
export function buildActionContext(
  source: ActionSource,
  overrides?: Partial<ActionExecutionContext>,
): ActionExecutionContext {
  const { tabs, activeTabId } = useWorkspaceStore.getState();
  const activeTab = tabs.find((t) => t.id === activeTabId);
  const explorerConnectionId =
    useConnectionStore.getState().explorerConnectionId;

  const connectionId =
    overrides?.connectionId ??
    activeTab?.connectionId ??
    explorerConnectionId ??
    undefined;

  const tabId = overrides?.tabId ?? activeTabId ?? undefined;

  let database: string | null | undefined;
  let schema: string | null | undefined;

  if (activeTab && activeTab.kind === "query") {
    database = activeTab.data.context.database;
    schema = activeTab.data.context.schema;
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
