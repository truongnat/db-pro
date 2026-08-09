import { create } from "zustand";
import { persist } from "zustand/middleware";

export type ThemeMode = "system" | "light" | "dark";

/** Resolve "system" to the actual visual mode using prefers-color-scheme. */
export function resolveThemeMode(mode: ThemeMode): "light" | "dark" {
  if (mode !== "system") return mode;
  if (typeof window === "undefined") return "dark";
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
          const resolved = resolveThemeMode(state.mode);
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
