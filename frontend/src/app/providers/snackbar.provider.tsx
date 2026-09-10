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
    success: useCallback((message: string) => toast.success(message), []),
    error: useCallback((message: string) => toast.error(message), []),
    warning: useCallback((message: string) => toast.warning(message), []),
    info: useCallback((message: string) => toast.info(message), []),
  };

  return (
    <SnackbarContext.Provider value={contextValue}>
      {children}
    </SnackbarContext.Provider>
  );
}
