import { useEffect, useSyncExternalStore } from "react";
import { useThemeStore, resolveThemeMode } from "@/commons/stores/theme.store";

interface ThemeProviderProps {
  children: React.ReactNode;
}

/** Subscribe to prefers-color-scheme media query for system mode. */
function subscribeToSystemTheme(cb: () => void) {
  const mql = window.matchMedia("(prefers-color-scheme: dark)");
  mql.addEventListener("change", cb);
  return () => mql.removeEventListener("change", cb);
}

function getSystemTheme() {
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

export function ThemeProvider({ children }: ThemeProviderProps) {
  const mode = useThemeStore((state) => state.mode);

  // Listen for OS theme changes when in "system" mode
  const systemTheme = useSyncExternalStore(subscribeToSystemTheme, getSystemTheme, getSystemTheme);

  const resolved = resolveThemeMode(mode === "system" ? (systemTheme as "light" | "dark") : mode === "light" ? "light" : "dark");

  useEffect(() => {
    document.documentElement.setAttribute("data-theme", resolved);
  }, [resolved]);

  // Set initial attribute synchronously to prevent flash (SSR-safe pattern)
  if (typeof document !== "undefined") {
    const current = document.documentElement.getAttribute("data-theme");
    if (current !== resolved) {
      document.documentElement.setAttribute("data-theme", resolved);
    }
  }

  return <>{children}</>;
}
