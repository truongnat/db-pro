import * as Dialog from "@radix-ui/react-dialog";

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
          className="fixed left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 w-full max-w-[400px] rounded-[var(--radius-lg)] border bg-[var(--color-surface)] p-6 shadow-xl"
          style={{ borderColor: "var(--color-border)" }}
        >
          <Dialog.Title
            className="text-lg font-semibold"
            style={{ color: "var(--color-text)" }}
          >
            {t("query.parameters")}
          </Dialog.Title>
          <Dialog.Description
            className="mt-3 text-sm"
            style={{ color: "var(--color-text-secondary)" }}
          >
            {t("query.parametersComingSoon")}
          </Dialog.Description>
          <div className="mt-4 flex justify-end">
            <button
              className="rounded-[var(--radius-sm)] border px-4 py-2 text-sm transition-colors hover:bg-[var(--color-bg)]"
              style={{
                borderColor: "var(--color-border)",
                color: "var(--color-text)",
              }}
              onClick={onClose}
            >
              {t("common.actions.close")}
            </button>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
