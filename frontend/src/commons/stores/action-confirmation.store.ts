import { create } from "zustand";
import { confirmAction, rejectConfirmation } from "../actions/bus";

import type { ActionConfirmation, ActionResult } from "../actions/types";

/**
 * Global Action Confirmation Store.
 *
 * Confirmation lifecycle is NOT owned by any individual component.
 * Instead, ALL Action UI clients (Toolbar, Keyboard, Command Palette, Agent)
 * surface confirmation through this single global store.
 *
 * The store uses a QUEUE internally. When a new confirmation arrives while
 * another is pending, the old one is automatically rejected (destroying its
 * prepared invocation) before the new one is enqueued. This ensures:
 *
 *   A confirmation NEVER disappears from UI while its prepared
 *   invocation remains executable.
 *
 * Flow:
 *   1. Any caller invokes executeAction() and gets confirmation_required
 *   2. Caller pushes the confirmation into this store via setPending()
 *   3. ActionConfirmationHost (mounted once in AppShell) renders the dialog
 *   4. User confirms → confirmAction(id) → frozen invocation executes
 *   5. User rejects → rejectConfirmation(id) → prepared invocation destroyed
 */
interface ActionConfirmationState {
  /** The currently visible confirmation (head of queue). */
  pending: ActionConfirmation | null;
  isConfirming: boolean;
  lastResult: ActionResult | null;

  /**
   * Push a new confirmation request.
   *
   * If another confirmation is currently pending, it is automatically
   * rejected (destroying its prepared invocation) before the new one
   * is enqueued. This prevents orphaned prepared invocations.
   */
  setPending: (confirmation: ActionConfirmation) => void;

  /**
   * Clear the pending confirmation without confirming.
   *
   * This REJECTS the prepared invocation — the destructive action
   * is NOT executed. Equivalent to user pressing Cancel.
   */
  clearPending: () => void;

  /** Confirm the pending action — executes the frozen prepared invocation. */
  confirm: () => Promise<ActionResult>;

  /** Reject the pending action — destroys the prepared invocation. */
  reject: () => void;
}

export const useActionConfirmationStore = create<ActionConfirmationState>()((set, get) => ({
  pending: null,
  isConfirming: false,
  lastResult: null,

  setPending: (confirmation) => {
    const current = get().pending;
    // If a different confirmation is pending, reject it first.
    // This destroys the old prepared invocation — the destructive
    // action can NEVER be executed after being replaced.
    if (current && current.id !== confirmation.id) {
      rejectConfirmation(current.id);
    }
    set({ pending: confirmation });
  },

  clearPending: () => {
    const current = get().pending;
    // Reject the prepared invocation before clearing UI state.
    // Hiding the confirmation WITHOUT rejecting would leave a
    // live prepared invocation in the bus — a safety violation.
    if (current) {
      rejectConfirmation(current.id);
    }
    set({ pending: null, isConfirming: false });
  },

  confirm: async () => {
    const { pending } = get();
    if (!pending) {
      return {
        status: "error" as const,
        error: { code: "no_confirmation", message: "No pending confirmation" },
      };
    }

    const confirmationId = pending.id;
    set({ isConfirming: true });

    try {
      const result = await confirmAction(confirmationId);
      set({ lastResult: result, pending: null, isConfirming: false });
      return result;
    } catch (err) {
      const message = err instanceof Error ? err.message : "Confirm failed";
      const result: ActionResult = {
        status: "error",
        error: { code: "confirm_failed", message },
      };
      set({ lastResult: result, pending: null, isConfirming: false });
      return result;
    }
  },

  reject: () => {
    const { pending } = get();
    if (!pending) return;
    rejectConfirmation(pending.id);
    set({ pending: null, isConfirming: false });
  },
}));
