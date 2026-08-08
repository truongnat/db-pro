import { create } from "zustand";
import { persist } from "zustand/middleware";

export type SidebarView = "explorer" | "search" | "query-saved" | "users";

const SIDEBAR_DEFAULT = 260;
const SIDEBAR_MIN = 220;
const SIDEBAR_MAX = 420;

const AGENT_DEFAULT = 320;
const AGENT_MIN = 260;
const AGENT_MAX = 500;

interface ShellState {
  sidebarCollapsed: boolean;
  sidebarView: SidebarView;
  sidebarWidth: number;
  agentOpen: boolean;
  agentWidth: number;
  toggleSidebar: () => void;
  setSidebarCollapsed: (collapsed: boolean) => void;
  setSidebarView: (view: SidebarView) => void;
  setSidebarWidth: (width: number) => void;
  toggleAgent: () => void;
  setAgentOpen: (open: boolean) => void;
  setAgentWidth: (width: number) => void;
}

export { SIDEBAR_DEFAULT, SIDEBAR_MIN, SIDEBAR_MAX, AGENT_DEFAULT, AGENT_MIN, AGENT_MAX };

export const useShellStore = create<ShellState>()(
  persist(
    (set) => ({
      sidebarCollapsed: false,
      sidebarView: "explorer" as SidebarView,
      sidebarWidth: SIDEBAR_DEFAULT,
      agentOpen: false,
      agentWidth: AGENT_DEFAULT,
      toggleSidebar: () => set((s) => ({ sidebarCollapsed: !s.sidebarCollapsed })),
      setSidebarCollapsed: (collapsed) => set({ sidebarCollapsed: collapsed }),
      setSidebarView: (view) => set({ sidebarView: view }),
      setSidebarWidth: (width) =>
        set({ sidebarWidth: Math.max(SIDEBAR_MIN, Math.min(SIDEBAR_MAX, width)) }),
      toggleAgent: () => set((s) => ({ agentOpen: !s.agentOpen })),
      setAgentOpen: (open) => set({ agentOpen: open }),
      setAgentWidth: (width) =>
        set({ agentWidth: Math.max(AGENT_MIN, Math.min(AGENT_MAX, width)) }),
    }),
    {
      name: "db-pro-shell",
    },
  ),
);
