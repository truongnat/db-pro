import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogCancel,
  AlertDialogAction,
} from "@/components/ui/alert-dialog";

import type { ActionConfirmation } from "@/commons/actions/types";
import { useTranslation } from "@/commons/locales/useTranslation";

interface ActionConfirmationDialogProps {
  confirmation: ActionConfirmation;
  onConfirm: () => void;
  onCancel: () => void;
}

/**
 * Generic confirmation dialog for destructive/risky actions.
 *
 * Wired to the Action Platform: when an action returns
 * `confirmation_required`, the UI renders this dialog.
 *
 * - Approve → calls confirmAction() which executes the frozen invocation
 * - Reject  → calls rejectConfirmation() which discards the prepared invocation
 */
export function ActionConfirmationDialog({
  confirmation,
  onConfirm,
  onCancel,
}: ActionConfirmationDialogProps) {
  const { t } = useTranslation();
  const isDestructive = confirmation.risk === "destructive";

  return (
    <AlertDialog
      open
      onOpenChange={(open) => {
        if (!open) onCancel();
      }}
    >
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>
            {isDestructive ? t("common.destructiveAction") : t("common.confirmAction")}
          </AlertDialogTitle>
          <AlertDialogDescription>{confirmation.message}</AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel onClick={onCancel}>{t("common.actions.cancel")}</AlertDialogCancel>
          <AlertDialogAction
            onClick={onConfirm}
            className={
              isDestructive
                ? "bg-destructive text-destructive-foreground hover:bg-destructive/90"
                : undefined
            }
          >
            {isDestructive ? t("common.executeAnyway") : t("common.actions.confirm")}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
