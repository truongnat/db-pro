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
 * Flow:
 *   1. Any caller invokes executeAction() and gets confirmation_required
 *   2. Caller (or the coordinator) pushes the confirmation into this store
 *   3. ActionConfirmationHost (mounted once in AppShell) renders the dialog
 *   4. User confirms → confirmAction(id) → frozen invocation executes
 *   5. User rejects → rejectConfirmation(id) → prepared invocation destroyed
 *
 * This replaces the old per-component local confirmation state
 * (e.g. useActionExecution in QueryTabContent).
 */
interface ActionConfirmationState {
  pending: ActionConfirmation | null;
  isConfirming: boolean;
  lastResult: ActionResult | null;

  /** Push a new confirmation request into the global queue. */
  setPending: (confirmation: ActionConfirmation) => void;

  /** Clear the pending confirmation without confirming or rejecting. */
  clearPending: () => void;

  /** Confirm the pending action — executes the frozen prepared invocation. */
  confirm: () => Promise<ActionResult>;

  /** Reject the pending action — destroys the prepared invocation. */
  reject: () => void;
}

export const useActionConfirmationStore =
  create<ActionConfirmationState>()((set, get) => ({
    pending: null,
    isConfirming: false,
    lastResult: null,

    setPending: (confirmation) => {
      set({ pending: confirmation });
    },

    clearPending: () => {
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
