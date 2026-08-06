import * as Dialog from "@radix-ui/react-dialog";

import { Button } from "@/components/ui/button";
import { useTranslation } from "@/commons/locales/useTranslation";

interface ParameterInputDialogProps {
  open: boolean;
  onClose: () => void;
}

export function ParameterInputDialog({
  open,
  onClose,
}: ParameterInputDialogProps) {
  const { t } = useTranslation();

  return (
    <Dialog.Root open={open} onOpenChange={(isOpen) => !isOpen && onClose()}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/50" />
        <Dialog.Content
          className="fixed left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 w-full max-w-[400px] rounded-lg border border-border bg-card p-6 shadow-xl"
        >
          <Dialog.Title className="text-lg font-semibold text-foreground">
            {t("query.parameters")}
          </Dialog.Title>
          <Dialog.Description className="mt-3 text-sm text-muted-foreground">
            {t("query.parametersComingSoon")}
          </Dialog.Description>
          <div className="mt-4 flex justify-end">
            <Button
              type="button"
              variant="outline"
              className="rounded-sm px-4 py-2 text-sm"
              onClick={onClose}
            >
              {t("common.actions.close")}
            </Button>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
