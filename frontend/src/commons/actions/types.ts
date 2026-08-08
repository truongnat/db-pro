// ─── Action ID ───────────────────────────────────────────────

/**
 * Dot-separated stable action identifier.
 *
 * Convention: <domain>.<verb>[.<qualifier>]
 *
 * Examples:
 *   query.execute.current
 *   workspace.tab.close
 *   explorer.openObject
 */
export type ActionId = string;

// ─── Source ──────────────────────────────────────────────────

/** Where the action invocation originated. */
export type ActionSource =
  | "ui"
  | "keyboard"
  | "command-palette"
  | "agent"
  | "mcp";

// ─── Risk ────────────────────────────────────────────────────

/**
 * Safety classification of an action.
 *
 * - read:        SELECT, refresh, export — no side effects
 * - write:       INSERT, UPDATE, ALTER — modifies data/schema
 * - destructive: DELETE, DROP, TRUNCATE — potentially irreversible
 */
export type ActionRisk = "read" | "write" | "destructive";

// ─── Category ────────────────────────────────────────────────

export type ActionCategory =
  | "query"
  | "workspace"
  | "explorer"
  | "connection"
  | "data"
  | "schema"
  | "agent"
  | "file"
  | "shell";

// ─── Execution Context ───────────────────────────────────────

/**
 * Carries provenance and ambient state for every action invocation.
 *
 * The `source` field lets downstream logic (audit, safety, logging)
 * know exactly where the request came from.
 */
export interface ActionExecutionContext {
  source: ActionSource;
  workspaceId?: string;
  tabId?: string;
  connectionId?: string;
  database?: string | null;
  schema?: string | null;
  correlationId: string;
  idempotencyKey?: string;
}

// ─── Availability ────────────────────────────────────────────

export interface ActionAvailability {
  status: "available" | "unavailable";
  reason?: string;
}

// ─── Confirmation ────────────────────────────────────────────

export interface ConfirmationPolicy {
  /** When "always", every invocation requires confirmation. */
  mode: "always" | "destructive-only";
  messageKey?: string;
}

// ─── Permission ──────────────────────────────────────────────

export interface ActionPermission {
  resource: string;
  action: string;
}

// ─── Schema wrapper ──────────────────────────────────────────

/**
 * Minimal contract that wraps a Zod schema.
 * Using an interface keeps the action core agnostic to the
 * exact Zod version / type name.
 */
export interface ActionSchema<T = unknown> {
  parse(data: unknown): T;
  safeParse(data: unknown): { success: boolean; data?: T; error?: unknown };
}

// ─── Action Definition ───────────────────────────────────────

/**
 * Full metadata + handler for a single registered action.
 *
 * This is the single source of truth for:
 *   - UI button disabled state  (via `availability`)
 *   - Command Palette entry     (via `title`, `category`)
 *   - MCP tool definition       (via `inputSchema`, `description`)
 *   - Safety policy             (via `risk`, `confirmation`)
 *   - Audit trail               (via `id`, `category`)
 */
export interface ActionDefinition<TInput = void, TOutput = unknown> {
  id: ActionId;
  title: string;
  description?: string;
  category: ActionCategory;

  inputSchema: ActionSchema<TInput>;
  outputSchema?: ActionSchema<TOutput>;

  execute(
    input: TInput,
    context: ActionExecutionContext,
  ): Promise<ActionResult<TOutput>>;

  availability?(
    context: ActionExecutionContext,
  ): ActionAvailability;

  /**
   * Optional cancel hook for long-running executions.
   * Called by the bus when cancelExecution() is invoked.
   */
  cancel?(
    execution: ActionExecution,
    context: ActionExecutionContext,
  ): Promise<void>;

  /**
   * Dynamic risk resolution based on actual input.
   * When present, overrides the static `risk` field.
   * Called after input validation but before confirmation gate.
   */
  resolveRisk?(
    input: TInput,
    context: ActionExecutionContext,
  ): ActionRisk;

  /**
   * Provides default input when invoked from the command palette.
   * Actions that require specific input (e.g. sql, name) should
   * define this so command palette invocations don't fail with
   * invalid_input.
   */
  commandInput?(): Partial<TInput> | undefined;

  risk?: ActionRisk;
  confirmation?: ConfirmationPolicy;
  permissions?: ActionPermission[];
}

// ─── Action Result ───────────────────────────────────────────

export interface ActionError {
  code: string;
  message: string;
  details?: unknown;
}

/**
 * Structured result returned by every action execution.
 *
 * Status values:
 *   success              — action completed normally
 *   error                — action failed
 *   cancelled            — action was cancelled (e.g. query cancel)
 *   confirmation_required — action needs user confirmation before proceeding
 */
export interface ActionResult<T = unknown> {
  status: "success" | "error" | "cancelled" | "confirmation_required";
  data?: T;
  error?: ActionError;
  effects?: ActionEffect[];
  executionId?: string;
  confirmation?: ActionConfirmation;
}

// ─── Effect ──────────────────────────────────────────────────

/**
 * Describes a state change that occurred as a result of an action.
 *
 * Effects let MCP/Agent understand how the application state changed
 * without having to re-query the entire workspace state.
 */
export interface ActionEffect {
  type: string;
  [key: string]: unknown;
}

// ─── Confirmation ────────────────────────────────────────────

export interface ActionConfirmation {
  id: string;
  actionId: ActionId;
  message: string;
  risk: ActionRisk;
  /** Original input to replay on confirm. */
  input?: Record<string, unknown>;
  /** Original context overrides to replay on confirm. */
  context?: Partial<ActionExecutionContext>;
  /** Original source of the invocation. */
  source?: ActionSource;
  /** When the confirmation was created. */
  createdAt: number;
  /** Present when the confirmation has been approved. */
  confirmed?: boolean;
  /** Who confirmed it (e.g. "user", "agent"). */
  confirmedBy?: string;
}

// ─── Audit ───────────────────────────────────────────────────

/**
 * Immutable record emitted at the start and completion of every
 * action execution.  Consumed by the audit log for traceability.
 */
export interface ActionAuditEvent {
  actionId: ActionId;
  source: ActionSource;
  startedAt: string;
  completedAt?: string;
  status: "started" | "completed" | "error" | "cancelled";
  connectionId?: string;
  correlationId: string;
  executionId?: string;
  confirmationId?: string;
  confirmedBy?: string;
  durationMs?: number;
}

// ─── Execution tracking ──────────────────────────────────────

export interface ActionExecution {
  executionId: string;
  actionId: ActionId;
  state: "running" | "completed" | "cancelled" | "error";
  progress?: ActionProgress;
  result?: ActionResult;
  startedAt: number;
}

export interface ActionProgress {
  completed: number;
  total: number;
  unit?: string;
  message?: string;
}

// ─── Resource reference ──────────────────────────────────────

/**
 * Canonical reference to a database resource.
 *
 * Instead of passing scattered parameters (connectionId, schema, name),
 * actions can accept a single ResourceRef for clarity and type safety.
 */
export type ResourceRef =
  | {
      type: "connection";
      connectionId: string;
    }
  | {
      type: "schema";
      connectionId: string;
      schema: string;
    }
  | {
      type: "table";
      connectionId: string;
      schema: string;
      name: string;
    }
  | {
      type: "view";
      connectionId: string;
      schema: string;
      name: string;
    }
  | {
      type: "query";
      tabId: string;
    };

// ─── Composite ID union (for exhaustive matching) ────────────

export type KnownActionId =
  // query
  | "query.execute.current"
  | "query.execute.selection"
  | "query.execute.all"
  | "query.cancel"
  | "query.explain"
  | "query.format"
  | "query.clear"
  | "query.save"
  | "query.get_context"
  | "query.get_sql"
  | "query.get_result"
  // workspace
  | "workspace.tab.open"
  | "workspace.tab.close"
  | "workspace.tab.activate"
  | "workspace.tab.pin"
  | "workspace.panel.set"
  | "workspace.get_state"
  // explorer
  | "explorer.refresh"
  | "explorer.toggleNode"
  | "explorer.expandNode"
  | "explorer.openObject"
  | "explorer.get_selection"
  // connection
  | "connection.new"
  | "connection.setActive"
  | "connection.list"
  // database
  | "database.connect"
  | "database.disconnect"
  | "database.reconnect"
  // data
  | "data.refresh"
  | "data.filter"
  | "data.sort"
  | "data.export"
  | "data.row.insert"
  | "data.row.update"
  | "data.row.delete"
  // schema
  | "schema.table.create"
  | "schema.table.alter"
  | "schema.table.drop"
  // agent
  | "agent.open"
  | "agent.close";
