import { z } from "zod";

import { getRegisteredActions } from "./registry";
import { buildActionContext } from "./context";

import type { ActionDefinition, ActionId, ActionRisk } from "./types";

/**
 * MCP Bridge — generates MCP tool definitions from the Action Registry.
 *
 * Instead of exposing a single generic `dbpro.execute_action(id, args)`
 * tool, we generate one MCP tool per action with a human-friendly name
 * and a JSON Schema derived from the action's inputSchema.
 *
 * This gives LLM agents full discoverability: they can list all
 * available tools and understand each one's parameters.
 */

// ─── MCP Tool interface ──────────────────────────────────────

/**
 * Minimal MCP tool definition.
 *
 * This is intentionally framework-agnostic. When you integrate with
 * a specific MCP SDK (e.g. @modelcontextprotocol/sdk), map these
 * objects to the SDK's tool registration API.
 */
export interface McpToolDefinition {
  name: string;
  description: string;
  inputSchema: Record<string, unknown>;
  /** The action ID this tool delegates to. */
  actionId: ActionId;
  /**
   * Risk level. "dynamic" means the risk depends on input content
   * (e.g. query.execute.* can be read or destructive depending on SQL).
   * Do NOT advertise dynamic actions as statically "read".
   */
  risk: ActionRisk;
  /** When risk is dynamic, this describes the policy. */
  riskPolicy?: string;
}

// ─── Action ID → MCP tool name mapping ───────────────────────

/**
 * Explicit overrides for tool names that should differ from the
 * auto-generated snake_case name.
 */
const TOOL_NAME_OVERRIDES: Partial<Record<ActionId, string>> = {
  "query.execute.current": "dbpro_execute_query",
  "query.execute.all": "dbpro_execute_all_queries",
  "query.execute.selection": "dbpro_execute_selection",
  "query.cancel": "dbpro_cancel_query",
  "query.explain": "dbpro_explain_query",
  "query.format": "dbpro_format_sql",
  "query.clear": "dbpro_clear_editor",
  "query.save": "dbpro_save_query",
  "workspace.tab.open": "dbpro_open_tab",
  "workspace.tab.close": "dbpro_close_tab",
  "workspace.tab.activate": "dbpro_activate_tab",
  "workspace.tab.pin": "dbpro_pin_tab",
  "workspace.panel.set": "dbpro_set_panel",
  "workspace.get_state": "dbpro_get_workspace_state",
  "explorer.refresh": "dbpro_refresh_explorer",
  "explorer.toggleNode": "dbpro_toggle_explorer_node",
  "explorer.expandNode": "dbpro_expand_explorer_node",
  "explorer.openObject": "dbpro_open_table",
  "explorer.get_selection": "dbpro_get_explorer_selection",
  "connection.new": "dbpro_new_connection",
  "connection.setActive": "dbpro_set_active_connection",
  "connection.list": "dbpro_list_connections",
  "data.refresh": "dbpro_refresh_table_data",
  "data.row.insert": "dbpro_insert_row",
  "data.row.update": "dbpro_update_row",
  "data.row.delete": "dbpro_delete_row",
  "schema.table.create": "dbpro_create_table",
  "schema.table.alter": "dbpro_alter_table",
  "schema.table.drop": "dbpro_drop_table",
  // query state
  "query.get_context": "dbpro_get_query_context",
  "query.get_sql": "dbpro_get_query_sql",
  "query.get_result": "dbpro_get_query_result",
  // database
  "database.connect": "dbpro_connect",
  "database.disconnect": "dbpro_disconnect",
  "database.reconnect": "dbpro_reconnect",
  // data extras
  "data.filter": "dbpro_filter_data",
  "data.sort": "dbpro_sort_data",
  "data.export": "dbpro_export_data",
  // agent
  "agent.open": "dbpro_open_agent",
  "agent.close": "dbpro_close_agent",
};

// ─── Name generation ─────────────────────────────────────────

/** Convert a dot-separated action ID to snake_case tool name. */
function actionIdToToolName(id: ActionId): string {
  return `dbpro_${id.replace(/\./g, "_")}`;
}

/** Get the MCP tool name for an action. */
export function getToolName(actionId: ActionId): string {
  return TOOL_NAME_OVERRIDES[actionId] ?? actionIdToToolName(actionId);
}

// ─── Zod → JSON Schema ──────────────────────────────────────

/**
 * Convert an action's inputSchema to a JSON Schema object.
 *
 * Uses Zod v4's `z.toJSONSchema()` static method for accurate conversion.
 * This ensures all Zod types (enums, records, optionals, etc.) are
 * correctly represented in the MCP tool definitions.
 */
function schemaToJsonSchema(schema: ActionDefinition["inputSchema"]): Record<string, unknown> {
  try {
    // Zod v4: z.toJSONSchema() is the official API.
    return z.toJSONSchema(schema as z.ZodType, { io: "input" });
  } catch {
    // Fallback: accept any object.
    return { type: "object", properties: {} };
  }
}

// ─── Tool generation ─────────────────────────────────────────

/** Generate an MCP tool definition from an action. */
export function actionToMcpTool(def: ActionDefinition): McpToolDefinition {
  const hasDynamicRisk = !!def.resolveRisk;
  return {
    name: getToolName(def.id),
    description: def.description ?? def.title,
    inputSchema: schemaToJsonSchema(def.inputSchema),
    actionId: def.id,
    // If the action has resolveRisk(), risk is dynamic — don't lie.
    risk: hasDynamicRisk ? "dynamic" : (def.risk ?? "read"),
    riskPolicy: hasDynamicRisk ? "sql" : undefined,
  };
}

/** Generate MCP tool definitions for all registered actions. */
export function generateMcpTools(): McpToolDefinition[] {
  return getRegisteredActions()
    .filter((action) => {
      // Exclude actions that are not implemented or currently unavailable.
      if (action.availability) {
        const ctx = buildActionContext("mcp");
        const avail = action.availability(ctx);
        if (avail.status === "unavailable" && avail.reason === "not_implemented") {
          return false;
        }
      }
      return true;
    })
    .map(actionToMcpTool);
}

/** Generate MCP tools filtered by category. */
export function generateMcpToolsByCategory(category: string): McpToolDefinition[] {
  return getRegisteredActions()
    .filter((a) => {
      if (a.category !== category) return false;
      if (a.availability) {
        const ctx = buildActionContext("mcp");
        const avail = a.availability(ctx);
        if (avail.status === "unavailable" && avail.reason === "not_implemented") {
          return false;
        }
      }
      return true;
    })
    .map(actionToMcpTool);
}

/**
 * Find the action ID for a given MCP tool name.
 *
 * Used by the MCP server to route incoming tool calls back to
 * the Action Platform.
 */
export function resolveToolToAction(toolName: string): ActionId | undefined {
  // Check overrides first (reverse lookup).
  for (const [actionId, name] of Object.entries(TOOL_NAME_OVERRIDES)) {
    if (name === toolName) return actionId;
  }

  // Fall back to convention-based lookup.
  const prefix = "dbpro_";
  if (!toolName.startsWith(prefix)) return undefined;

  const actionId = toolName.slice(prefix.length).replace(/_/g, ".");
  return getRegisteredActions().some((a) => a.id === actionId) ? actionId : undefined;
}

// ─── Generic fallback tool ───────────────────────────────────

/**
 * The generic `dbpro_execute_action` tool for advanced/debug use.
 *
 * This is NOT the primary MCP surface — individual tools are preferred
 * for agent discoverability. But this escape hatch is useful for
 * debugging and for actions that haven't been given a dedicated tool yet.
 */
export const GENERIC_TOOL: McpToolDefinition = {
  name: "dbpro_execute_action",
  description:
    "Execute any registered action by ID with arbitrary arguments. " +
    "Prefer using the specific dbpro_* tools for better discoverability.",
  inputSchema: {
    type: "object",
    properties: {
      actionId: {
        type: "string",
        description: "The stable action identifier (e.g. query.execute.current)",
      },
      args: {
        type: "object",
        description: "Action-specific arguments",
      },
    },
    required: ["actionId"],
  },
  actionId: "__generic__",
  risk: "read",
};
