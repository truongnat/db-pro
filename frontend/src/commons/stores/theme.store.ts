import { useSyncExternalStore } from "react";
import { create } from "zustand";
import { persist } from "zustand/middleware";

export type ThemeMode = "system" | "light" | "dark";
export type ResolvedTheme = "light" | "dark";

/** Resolve "system" to the actual visual mode using prefers-color-scheme. */
export function resolveThemeMode(mode: ThemeMode): ResolvedTheme {
  if (mode !== "system") return mode;
  if (typeof window === "undefined") return "dark";
  if (typeof window.matchMedia !== "function") return "dark";
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

interface ThemeState {
  mode: ThemeMode;
  setMode: (mode: ThemeMode) => void;
  toggle: () => void;
}

export const useThemeStore = create<ThemeState>()(
  persist(
    (set) => ({
      mode: "system",
      setMode: (mode) => set({ mode }),
      toggle: () =>
        set((state) => {
          // Cycle: system → light → dark → system
          if (state.mode === "system") return { mode: "light" };
          if (state.mode === "light") return { mode: "dark" };
          return { mode: "system" };
        }),
    }),
    {
      name: "db-pro-theme",
    },
  ),
);

function subscribeToTheme(cb: () => void) {
  const unsubStore = useThemeStore.subscribe(cb);
  let cleanup: (() => void) | undefined;
  if (typeof window !== "undefined" && typeof window.matchMedia === "function") {
    const mql = window.matchMedia("(prefers-color-scheme: dark)");
    mql.addEventListener("change", cb);
    cleanup = () => mql.removeEventListener("change", cb);
  }
  return () => {
    unsubStore();
    cleanup?.();
  };
}

function getResolvedTheme(): ResolvedTheme {
  return resolveThemeMode(useThemeStore.getState().mode);
}

/** Single reactive source for the resolved theme — reacts to both store changes and OS theme changes. */
export function useResolvedTheme(): ResolvedTheme {
  return useSyncExternalStore(subscribeToTheme, getResolvedTheme, getResolvedTheme);
}
