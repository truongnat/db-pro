import { create } from "zustand";
import { persist } from "zustand/middleware";

export type SidebarView = "explorer" | "search" | "query-saved" | "users";

const SIDEBAR_DEFAULT = 260;
const SIDEBAR_MIN = 220;
const SIDEBAR_MAX = 420;

interface ShellState {
  sidebarCollapsed: boolean;
  sidebarView: SidebarView;
  sidebarWidth: number;
  agentOpen: boolean;
  toggleSidebar: () => void;
  setSidebarCollapsed: (collapsed: boolean) => void;
  setSidebarView: (view: SidebarView) => void;
  setSidebarWidth: (width: number) => void;
  toggleAgent: () => void;
  setAgentOpen: (open: boolean) => void;
}

export { SIDEBAR_DEFAULT, SIDEBAR_MIN, SIDEBAR_MAX };

export const useShellStore = create<ShellState>()(
  persist(
    (set) => ({
      sidebarCollapsed: false,
      sidebarView: "explorer" as SidebarView,
      sidebarWidth: SIDEBAR_DEFAULT,
      agentOpen: false,
      toggleSidebar: () => set((s) => ({ sidebarCollapsed: !s.sidebarCollapsed })),
      setSidebarCollapsed: (collapsed) => set({ sidebarCollapsed: collapsed }),
      setSidebarView: (view) => set({ sidebarView: view }),
      setSidebarWidth: (width) =>
        set({ sidebarWidth: Math.max(SIDEBAR_MIN, Math.min(SIDEBAR_MAX, width)) }),
      toggleAgent: () => set((s) => ({ agentOpen: !s.agentOpen })),
      setAgentOpen: (open) => set({ agentOpen: open }),
    }),
    {
      name: "db-pro-shell",
    },
  ),
);
