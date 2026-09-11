import { type ReactNode, createContext, useCallback, useContext } from "react";

import { toast } from "sonner";

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

export function SnackbarProvider({ children }: { children: ReactNode }) {
  const contextValue: SnackbarContextValue = {
    success: useCallback((message: string) => toast.success(message, { duration: 4000 }), []),
    error: useCallback((message: string) => toast.error(message, { duration: Infinity }), []),
    warning: useCallback((message: string) => toast.warning(message, { duration: 6000 }), []),
    info: useCallback((message: string) => toast.info(message, { duration: 4000 }), []),
  };

  return <SnackbarContext.Provider value={contextValue}>{children}</SnackbarContext.Provider>;
}
