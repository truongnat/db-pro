import { ActionConfirmationDialog } from "./action-confirmation-dialog";
import { useActionConfirmationStore } from "@/commons/stores/action-confirmation.store";

/**
 * Global Action Confirmation Host.
 *
 * Mounted ONCE in AppShell. Renders the confirmation dialog whenever
 * the global Action Confirmation Store has a pending confirmation.
 *
 * ALL Action UI clients (Toolbar, Keyboard, Command Palette, Agent, MCP)
 * surface confirmation through the same host — no per-component renderers.
 *
 * Flow:
 *   executeAction() → confirmation_required
 *     → useActionConfirmationStore.setPending(confirmation)
 *       → Host renders ActionConfirmationDialog
 *         → Confirm → confirmAction(id)
 *         → Reject  → rejectConfirmation(id) (destroys prepared invocation)
 */
export function ActionConfirmationHost() {
  const pending = useActionConfirmationStore((s) => s.pending);
  const confirm = useActionConfirmationStore((s) => s.confirm);
  const reject = useActionConfirmationStore((s) => s.reject);

  if (!pending) return null;

  return (
    <ActionConfirmationDialog
      confirmation={pending}
      onConfirm={() => { void confirm(); }}
      onCancel={reject}
    />
  );
}
