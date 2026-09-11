import { type ReactNode, useCallback, useContext, createContext } from "react";

import { create } from "zustand";
import { useShallow } from "zustand/react/shallow";

import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogAction,
  AlertDialogCancel,
} from "@/components/ui/alert-dialog";

interface ConfirmState {
  open: boolean;
  title: string;
  message: string;
  confirmLabel: string;
  cancelLabel: string;
  resolve: ((value: boolean) => void) | null;
}

interface ConfirmStore extends ConfirmState {
  show: (opts: {
    title: string;
    message: string;
    confirmLabel?: string;
    cancelLabel?: string;
  }) => Promise<boolean>;
  setResult: (value: boolean) => void;
}

const useConfirmStore = create<ConfirmStore>()((set, get) => ({
  open: false,
  title: "",
  message: "",
  confirmLabel: "Confirm",
  cancelLabel: "Cancel",
  resolve: null,

  show: (opts) => {
    return new Promise<boolean>((resolve) => {
      set({
        open: true,
        title: opts.title,
        message: opts.message,
        confirmLabel: opts.confirmLabel ?? "Confirm",
        cancelLabel: opts.cancelLabel ?? "Cancel",
        resolve,
      });
    });
  },

  setResult: (value) => {
    const { resolve } = get();
    set({ open: false, resolve: null });
    resolve?.(value);
  },
}));

interface ConfirmContextValue {
  confirm: (opts: {
    title: string;
    message: string;
    confirmLabel?: string;
    cancelLabel?: string;
  }) => Promise<boolean>;
}

const ConfirmContext = createContext<ConfirmContextValue | null>(null);

export function useConfirmDialog(): ConfirmContextValue {
  const ctx = useContext(ConfirmContext);
  if (!ctx) {
    throw new Error("useConfirmDialog must be used within ConfirmDialogProvider");
  }
  return ctx;
}

export function ConfirmDialogProvider({ children }: { children: ReactNode }) {
  const show = useConfirmStore((s) => s.show);

  const confirm = useCallback(
    (opts: { title: string; message: string; confirmLabel?: string; cancelLabel?: string }) =>
      show(opts),
    [show],
  );

  return (
    <ConfirmContext.Provider value={{ confirm }}>
      {children}
      <ConfirmDialogHost />
    </ConfirmContext.Provider>
  );
}

function ConfirmDialogHost() {
  const { open, title, message, confirmLabel, cancelLabel, setResult } = useConfirmStore(
    useShallow((s) => ({
      open: s.open,
      title: s.title,
      message: s.message,
      confirmLabel: s.confirmLabel,
      cancelLabel: s.cancelLabel,
      setResult: s.setResult,
    })),
  );

  return (
    <AlertDialog
      open={open}
      onOpenChange={(isOpen) => {
        if (!isOpen) setResult(false);
      }}
    >
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{title}</AlertDialogTitle>
          <AlertDialogDescription>{message}</AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel onClick={() => setResult(false)}>{cancelLabel}</AlertDialogCancel>
          <AlertDialogAction onClick={() => setResult(true)}>{confirmLabel}</AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
