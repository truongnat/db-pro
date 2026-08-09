import type { ActionCategory, ActionDefinition, ActionId } from "./types";

/**
 * Central registry of all action definitions.
 *
 * Actions are registered at application bootstrap via `defineAction()`.
 * The registry is read-only after registration — there is no `unregister`.
 */

const actions = new Map<ActionId, ActionDefinition>();

/**
 * Register an action definition and return it unchanged.
 *
 * Usage:
 * ```ts
 * const executeCurrent = defineAction({
 *   id: "query.execute.current",
 *   title: "Run current statement",
 *   ...
 * });
 * ```
 *
 * Duplicate IDs overwrite with a console warning.
 */
export function defineAction<TInput, TOutput>(
  definition: ActionDefinition<TInput, TOutput>,
): ActionDefinition<TInput, TOutput> {
  if (actions.has(definition.id)) {
    console.warn(`[ActionRegistry] Overwriting action "${definition.id}"`);
  }
  actions.set(definition.id, definition as ActionDefinition);
  return definition;
}

/** Get all registered action definitions. */
export function getRegisteredActions(): readonly ActionDefinition[] {
  return Array.from(actions.values());
}

/** Look up a single action by ID. */
export function getAction(id: ActionId): ActionDefinition | undefined {
  return actions.get(id);
}

/** Get all actions in a given category. */
export function getActionsByCategory(category: ActionCategory): ActionDefinition[] {
  return Array.from(actions.values()).filter((a) => a.category === category);
}

/** Get all actions that carry a specific risk level. */
export function getActionsByRisk(risk: string): ActionDefinition[] {
  return Array.from(actions.values()).filter((a) => a.risk === risk);
}

/** Reset the registry (test only). */
export function resetActionRegistry(): void {
  actions.clear();
}
