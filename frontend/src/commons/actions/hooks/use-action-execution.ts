import { useState, useCallback, useRef } from "react";
import { executeAction, confirmAction, rejectConfirmation } from "../bus";

import type { ActionConfirmation, ActionId, ActionResult } from "../types";

/**
 * React hook for executing actions through the Action Platform.
 *
 * Handles the full lifecycle:
 *   1. Call executeAction() with the action ID and input
 *   2. If confirmation_required → expose pending confirmation
 *   3. On confirm → call confirmAction() with the frozen invocation
 *   4. On reject → call rejectConfirmation()
 *
 * Usage:
 *   const { execute, pendingConfirmation, confirm, reject } = useActionExecution();
 *   const result = await execute("query.execute.current", { tabId });
 *   if (result.status === "confirmation_required") { ... show dialog ... }
 */
export function useActionExecution() {
  const [pendingConfirmation, setPendingConfirmation] = useState<ActionConfirmation | null>(null);
  const [isExecuting, setIsExecuting] = useState(false);
  const lastResultRef = useRef<ActionResult | null>(null);

  const execute = useCallback(
    async (
      actionId: ActionId,
      input?: Record<string, unknown>,
      options?: { source?: "ui" | "keyboard" | "command-palette" },
    ): Promise<ActionResult> => {
      setIsExecuting(true);
      try {
        const result = await executeAction(actionId, input, {
          source: options?.source ?? "ui",
        });
        lastResultRef.current = result;

        if (result.status === "confirmation_required" && result.confirmation) {
          setPendingConfirmation(result.confirmation);
        }

        return result;
      } finally {
        setIsExecuting(false);
      }
    },
    [],
  );

  const confirm = useCallback(async (): Promise<ActionResult> => {
    if (!pendingConfirmation) {
      return {
        status: "error",
        error: { code: "no_confirmation", message: "No pending confirmation" },
      };
    }

    const confirmationId = pendingConfirmation.id;
    setPendingConfirmation(null);
    setIsExecuting(true);

    try {
      const result = await confirmAction(confirmationId);
      lastResultRef.current = result;
      return result;
    } finally {
      setIsExecuting(false);
    }
  }, [pendingConfirmation]);

  const reject = useCallback(() => {
    if (!pendingConfirmation) return;
    rejectConfirmation(pendingConfirmation.id);
    setPendingConfirmation(null);
  }, [pendingConfirmation]);

  return {
    execute,
    confirm,
    reject,
    pendingConfirmation,
    isExecuting,
    lastResult: lastResultRef.current,
  };
}
