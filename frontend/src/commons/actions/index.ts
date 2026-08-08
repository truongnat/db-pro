/**
 * Action Platform — bootstrap.
 *
 * Importing this module registers all action definitions.
 * Call this once during application startup (e.g. in main.tsx).
 */

// Core
export { defineAction, getAction, getRegisteredActions, getActionsByCategory, getActionsByRisk, resetActionRegistry } from "./registry";
export { executeAction, confirmAction, rejectConfirmation, getPendingConfirmations, isActionAvailable, getExecution, updateExecutionProgress, cancelExecution, cleanupExecutions } from "./bus";
export { buildActionContext, generateCorrelationId } from "./context";
export { onAuditEvent, emitAuditEvent, getAuditBuffer, clearAuditBuffer } from "./audit";

// Types
export type {
  ActionId,
  ActionSource,
  ActionRisk,
  ActionCategory,
  ActionExecutionContext,
  ActionAvailability,
  ConfirmationPolicy,
  ActionPermission,
  ActionSchema,
  ActionDefinition,
  ActionError,
  ActionResult,
  ActionEffect,
  ActionConfirmation,
  ActionAuditEvent,
  ActionExecution,
  ActionProgress,
  KnownActionId,
  ResourceRef,
} from "./types";

// Adapters
export { commandFromAction, commandsFromCategory, commandsFromAllActions, buildCommandContext } from "./command-adapter";
export { generateMcpTools, generateMcpToolsByCategory, actionToMcpTool, getToolName, resolveToolToAction, GENERIC_TOOL } from "./mcp-bridge";
export type { McpToolDefinition } from "./mcp-bridge";

// ─── Register all action definitions ─────────────────────────
// Side-effect imports: each module calls defineAction() at import time.

import "./definitions/query.actions";
import "./definitions/workspace.actions";
import "./definitions/explorer.actions";
import "./definitions/connection.actions";
import "./definitions/database.actions";
import "./definitions/data.actions";
import "./definitions/schema.actions";
import "./definitions/agent.actions";
