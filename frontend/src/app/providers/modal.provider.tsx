import {
  type ComponentType,
  type ReactNode,
  createContext,
  useCallback,
  useContext,
} from "react";

import * as Dialog from "@radix-ui/react-dialog";
import { create } from "zustand";

interface ModalEntry {
  id: string;
  component: ComponentType<{ onClose: () => void }>;
  props: Record<string, unknown>;
}

interface ModalState {
  modals: ModalEntry[];
  open: (
    id: string,
    component: ComponentType<{ onClose: () => void }>,
    props?: Record<string, unknown>,
  ) => void;
  close: (id: string) => void;
  closeAll: () => void;
}

const useModalStore = create<ModalState>()((set) => ({
  modals: [],

  open: (id, component, props = {}) =>
    set((state) => {
      const existing = state.modals.filter((m) => m.id !== id);
      return {
        modals: [...existing, { id, component, props }],
      };
    }),

  close: (id) =>
    set((state) => ({
      modals: state.modals.filter((m) => m.id !== id),
    })),

  closeAll: () => set({ modals: [] }),
}));

interface ModalContextValue {
  open: (
    id: string,
    component: ComponentType<{ onClose: () => void }>,
    props?: Record<string, unknown>,
  ) => void;
  close: (id: string) => void;
  closeAll: () => void;
}

const ModalContext = createContext<ModalContextValue | null>(null);

export function useModal(): ModalContextValue {
  const ctx = useContext(ModalContext);
  if (!ctx) {
    throw new Error("useModal must be used within ModalProvider");
  }
  return ctx;
}

export function ModalProvider({ children }: { children: ReactNode }) {
  const { modals, open, close, closeAll } = useModalStore();

  const contextValue: ModalContextValue = {
    open: useCallback(
      (
        id: string,
        component: ComponentType<{ onClose: () => void }>,
        props?: Record<string, unknown>,
      ) => open(id, component, props),
      [open],
    ),
    close: useCallback((id: string) => close(id), [close]),
    closeAll: useCallback(() => closeAll(), [closeAll]),
  };

  return (
    <ModalContext.Provider value={contextValue}>
      {children}
      {modals.map((entry) => {
        const ModalComponent = entry.component;
        return (
          <Dialog.Root
            key={entry.id}
            open
            onOpenChange={(isOpen) => {
              if (!isOpen) close(entry.id);
            }}
          >
            <Dialog.Portal>
              <Dialog.Overlay className="fixed inset-0 bg-black/50 animate-in fade-in duration-200" />
              <Dialog.Content className="fixed left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 bg-background border border-[var(--app-border)] rounded-lg shadow-xl max-w-[600px] w-full max-h-[80vh] overflow-auto animate-in fade-in zoom-in-95 duration-200">
                <ModalComponent
                  {...entry.props}
                  onClose={() => close(entry.id)}
                />
              </Dialog.Content>
            </Dialog.Portal>
          </Dialog.Root>
        );
      })}
    </ModalContext.Provider>
  );
}
