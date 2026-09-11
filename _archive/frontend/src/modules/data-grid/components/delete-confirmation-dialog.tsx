import { useTranslation } from "@/commons/locales/useTranslation";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";

interface DeleteConfirmationDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onConfirm: () => void;
  deleteCount: number;
  totalChanges: number;
}

export function DeleteConfirmationDialog({
  open,
  onOpenChange,
  onConfirm,
  deleteCount,
  totalChanges,
}: DeleteConfirmationDialogProps) {
  const { t } = useTranslation();

  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{t("dataGrid.confirmDeleteTitle")}</AlertDialogTitle>
          <AlertDialogDescription>
            {t("dataGrid.confirmDeleteDescription", {
              deleteCount,
              totalCount: totalChanges,
            })}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>{t("common.actions.cancel")}</AlertDialogCancel>
          <AlertDialogAction onClick={onConfirm}>
            {t("dataGrid.confirmDeleteAction")}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
