import type { Command } from "@/commons/types/command.types";
import { getAction, getRegisteredActions } from "./registry";
import { executeAction, isActionAvailable } from "./bus";
import { buildActionContext } from "./context";
import { useActionConfirmationStore } from "@/commons/stores/action-confirmation.store";

import type { ActionId } from "./types";

/**
 * Bridge between the legacy Command system and the Action Platform.
 *
 * Each ActionDefinition can be projected into a Command that the
 * existing CommandPalette and keyboard handler understand.
 *
 * This lets us migrate one action at a time: register it in the
 * Action Platform, then replace the old hand-written Command entry
 * with `commandFromAction(id)`.
 */

/**
 * Convert an ActionDefinition into a Command for the command palette.
 *
 * The generated Command:
 *   - Uses the action's `availability()` for the `when()` guard
 *   - Delegates `execute()` to the action bus with source = "command-palette"
 *   - Derives `labelKey` from the action title (convention: "actions.<id>")
 *
 * Input resolution order:
 *   1. Explicit `inputProvider` argument (caller-supplied)
 *   2. Action's own `commandInput()` hook
 *   3. undefined (zero-input actions)
 */
export function commandFromAction(
  actionId: ActionId,
  options?: { inputProvider?: () => Record<string, unknown> | undefined },
): Command {
  const def = getAction(actionId);
  if (!def) {
    throw new Error(`[commandFromAction] Action "${actionId}" is not registered`);
  }

  return {
    id: def.id,
    // Convention: "actions.<actionId>" → i18n key.
    // Falls back to the action title if no translation exists.
    labelKey: `actions.${def.id}`,
    groupKey: `commands.groups.${def.category}`,
    when: () => isActionAvailable(def.id).available,
    execute: async () => {
      const input = options?.inputProvider?.() ?? def.commandInput?.() ?? undefined;
      const result = await executeAction(def.id, input as Record<string, unknown> | undefined, {
        source: "command-palette",
      });

      // Route confirmation to the global Action Confirmation Host.
      // This ensures destructive actions from Command Palette show the
      // same confirmation dialog as toolbar/keyboard invocations.
      if (result.status === "confirmation_required" && result.confirmation) {
        useActionConfirmationStore.getState().setPending(result.confirmation);
      }
    },
  };
}

/**
 * Generate Command entries for all registered actions in a category.
 *
 * Useful for bulk-migrating a domain (e.g. all query actions) at once.
 */
export function commandsFromCategory(category: string): Command[] {
  return getRegisteredActions()
    .filter((a) => a.category === category)
    .map((a) => commandFromAction(a.id));
}

/**
 * Generate Command entries for ALL registered actions.
 *
 * Call this during bootstrap to replace the manual `registerMany([...])`
 * call in register-commands.ts.
 */
export function commandsFromAllActions(): Command[] {
  return getRegisteredActions().map((a) => commandFromAction(a.id));
}

/**
 * Build a context pre-populated for command-palette / keyboard source.
 *
 * Helper for the command adapter to avoid repeating the source literal.
 */
export function buildCommandContext(overrides?: Partial<ReturnType<typeof buildActionContext>>) {
  return buildActionContext("command-palette", overrides);
}
