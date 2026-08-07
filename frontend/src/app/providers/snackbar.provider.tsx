import {
  type ReactNode,
  createContext,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
} from "react";

import { create } from "zustand";

type SnackbarVariant = "success" | "error" | "warning" | "info";

interface SnackbarItem {
  id: string;
  variant: SnackbarVariant;
  message: string;
  duration: number;
}

interface SnackbarState {
  items: SnackbarItem[];
  add: (variant: SnackbarVariant, message: string) => void;
  dismiss: (id: string) => void;
}

const DURATIONS: Record<SnackbarVariant, number> = {
  success: 4000,
  error: 8000,
  warning: 6000,
  info: 4000,
};

const MAX_ITEMS = 3;

const useSnackbarStore = create<SnackbarState>()((set) => ({
  items: [],

  add: (variant, message) =>
    set((state) => {
      const newItem: SnackbarItem = {
        id: crypto.randomUUID(),
        variant,
        message,
        duration: DURATIONS[variant],
      };
      const items = [...state.items, newItem].slice(-MAX_ITEMS);
      return { items };
    }),

  dismiss: (id) =>
    set((state) => ({
      items: state.items.filter((item) => item.id !== id),
    })),
}));

interface SnackbarContextValue {
  success: (message: string) => void;
  error: (message: string) => void;
  warning: (message: string) => void;
  info: (message: string) => void;
}

const SnackbarContext = createContext<SnackbarContextValue | null>(null);

export function useSnackbar(): SnackbarContextValue {
  const ctx = useContext(SnackbarContext);
  if (!ctx) {
    throw new Error("useSnackbar must be used within SnackbarProvider");
  }
  return ctx;
}

const VARIANT_STYLES: Record<SnackbarVariant, string> = {
  success: "bg-success text-white",
  error: "bg-destructive text-white",
  warning: "bg-warning text-white",
  info: "bg-info text-white",
};

function SnackbarItemComponent({
  item,
  onDismiss,
}: {
  item: SnackbarItem;
  onDismiss: (id: string) => void;
}) {
  const timerRef = useRef<ReturnType<typeof setTimeout>>(null);
  const [isPaused, setIsPaused] = useState(false);

  useEffect(() => {
    if (!isPaused) {
      timerRef.current = setTimeout(() => {
        onDismiss(item.id);
      }, item.duration);
    }

    return () => {
      if (timerRef.current) {
        clearTimeout(timerRef.current);
      }
    };
  }, [item.id, item.duration, isPaused, onDismiss]);

  return (
    <div
      role="alert"
      aria-live="polite"
      className={`px-4 py-3 rounded-md shadow-lg min-w-[300px] max-w-[500px] cursor-pointer transition-all duration-200 ${VARIANT_STYLES[item.variant]}`}
      onClick={() => onDismiss(item.id)}
      onMouseEnter={() => setIsPaused(true)}
      onMouseLeave={() => setIsPaused(false)}
    >
      {item.message}
    </div>
  );
}

export function SnackbarProvider({ children }: { children: ReactNode }) {
  const { items, dismiss, add } = useSnackbarStore();

  const contextValue: SnackbarContextValue = {
    success: useCallback(
      (message: string) => add("success", message),
      [add],
    ),
    error: useCallback((message: string) => add("error", message), [add]),
    warning: useCallback(
      (message: string) => add("warning", message),
      [add],
    ),
    info: useCallback((message: string) => add("info", message), [add]),
  };

  return (
    <SnackbarContext.Provider value={contextValue}>
      {children}
      <div
        className="fixed bottom-4 right-4 flex flex-col gap-2 z-[var(--z-toast)]"
        aria-label="Notifications"
      >
        {items.map((item) => (
          <SnackbarItemComponent
            key={item.id}
            item={item}
            onDismiss={dismiss}
          />
        ))}
      </div>
    </SnackbarContext.Provider>
  );
}
