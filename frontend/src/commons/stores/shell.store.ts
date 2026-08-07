import { create } from "zustand";
import { persist } from "zustand/middleware";

export type SidebarView = "explorer" | "search" | "query-saved" | "connections" | "users";

interface ShellState {
  sidebarCollapsed: boolean;
  sidebarView: SidebarView;
  toggleSidebar: () => void;
  setSidebarCollapsed: (collapsed: boolean) => void;
  setSidebarView: (view: SidebarView) => void;
}

export const useShellStore = create<ShellState>()(
  persist(
    (set) => ({
      sidebarCollapsed: false,
      sidebarView: "explorer" as SidebarView,
      toggleSidebar: () => set((s) => ({ sidebarCollapsed: !s.sidebarCollapsed })),
      setSidebarCollapsed: (collapsed) => set({ sidebarCollapsed: collapsed }),
      setSidebarView: (view) => set({ sidebarView: view }),
    }),
    {
      name: "db-pro-shell",
    },
  ),
);
