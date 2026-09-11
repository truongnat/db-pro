import { useEffect } from "react";
import { useResolvedTheme } from "@/commons/stores/theme.store";

interface ThemeProviderProps {
  children: React.ReactNode;
}

export function ThemeProvider({ children }: ThemeProviderProps) {
  const resolved = useResolvedTheme();

  useEffect(() => {
    document.documentElement.setAttribute("data-theme", resolved);
  }, [resolved]);

  if (typeof document !== "undefined") {
    const current = document.documentElement.getAttribute("data-theme");
    if (current !== resolved) {
      document.documentElement.setAttribute("data-theme", resolved);
    }
  }

  return <>{children}</>;
}
