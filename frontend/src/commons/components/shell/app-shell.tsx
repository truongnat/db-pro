import { useCallback, useRef } from "react";

import { CommandPalette } from "@/commons/components/command-palette";
import { QuickOpen } from "@/commons/components/quick-open";
import { WorkspaceContent } from "@/commons/components/workspace-content";
import { WorkspaceTabBar } from "@/commons/components/workspace-tab-bar";
import { useCommandPalette } from "@/commons/hooks/use-command-palette";
import { useQuickOpen } from "@/commons/hooks/use-quick-open";
import { useShellStore } from "@/commons/stores/shell.store";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { useRegisterRuntimeCacheInvalidation } from "@/modules/query/queries/query.queries";
import { ConnectionDialog } from "@/modules/connection/components/connection-dialog";
import { ActionConfirmationHost } from "../action-confirmation-host";

import { ActivityBar } from "./activity-bar";
import { AgentPanel } from "../ide/agent-panel";
import { Sidebar } from "./sidebar";
import { StatusBar } from "./status-bar";
import { Topbar } from "./topbar";

export function AppShell() {
  const sidebarCollapsed = useShellStore((s) => s.sidebarCollapsed);
  const sidebarWidth = useShellStore((s) => s.sidebarWidth);
  const setSidebarWidth = useShellStore((s) => s.setSidebarWidth);
  const agentOpen = useShellStore((s) => s.agentOpen);
  const setAgentOpen = useShellStore((s) => s.setAgentOpen);
  const agentWidth = useShellStore((s) => s.agentWidth);
  const setAgentWidth = useShellStore((s) => s.setAgentWidth);
  const hasTabs = useWorkspaceStore((s) => s.tabs.length > 0);
  const draggingRef = useRef(false);
  useCommandPalette();
  useQuickOpen();
  // Register TanStack Query cache invalidation with the canonical runtime
  // so that ALL execution sources (UI, Action, MCP) trigger history refresh.
  useRegisterRuntimeCacheInvalidation();

  const handleSidebarResizeStart = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault();
      draggingRef.current = true;
      document.body.style.cursor = "col-resize";
      document.body.style.userSelect = "none";

      const startX = e.clientX;
      const startWidth = sidebarWidth;

      const onMouseMove = (moveEvent: MouseEvent) => {
        if (!draggingRef.current) return;
        const delta = moveEvent.clientX - startX;
        setSidebarWidth(startWidth + delta);
      };

      const onMouseUp = () => {
        draggingRef.current = false;
        document.body.style.cursor = "";
        document.body.style.userSelect = "";
        document.removeEventListener("mousemove", onMouseMove);
        document.removeEventListener("mouseup", onMouseUp);
      };

      document.addEventListener("mousemove", onMouseMove);
      document.addEventListener("mouseup", onMouseUp);
    },
    [sidebarWidth, setSidebarWidth],
  );

  const handleAgentResizeStart = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault();
      draggingRef.current = true;
      document.body.style.cursor = "col-resize";
      document.body.style.userSelect = "none";

      const startX = e.clientX;
      const startWidth = agentWidth;

      const onMouseMove = (moveEvent: MouseEvent) => {
        if (!draggingRef.current) return;
        // Agent is on the right: dragging left increases width
        const delta = startX - moveEvent.clientX;
        setAgentWidth(startWidth + delta);
      };

      const onMouseUp = () => {
        draggingRef.current = false;
        document.body.style.cursor = "";
        document.body.style.userSelect = "";
        document.removeEventListener("mousemove", onMouseMove);
        document.removeEventListener("mouseup", onMouseUp);
      };

      document.addEventListener("mousemove", onMouseMove);
      document.addEventListener("mouseup", onMouseUp);
    },
    [agentWidth, setAgentWidth],
  );

  const effectiveSidebarWidth = sidebarCollapsed ? 0 : sidebarWidth;

  // Grid columns: sidebar [resize] main [resize] agent
  const contentColumns =
    sidebarCollapsed && !agentOpen
      ? "1fr"
      : sidebarCollapsed
        ? `1fr 3px ${agentWidth}px`
        : agentOpen
          ? `${effectiveSidebarWidth}px 3px minmax(0, 1fr) 3px ${agentWidth}px`
          : `${effectiveSidebarWidth}px 3px minmax(0, 1fr)`;

  return (
    <>
      <div
        className="grid h-screen overflow-hidden bg-[var(--surface-app)]"
        style={{ gridTemplateColumns: "var(--app-activity-bar-width) 1fr" }}
      >
        <ActivityBar />

        <div
          className="grid min-h-0 min-w-0"
          style={{ gridTemplateRows: "var(--app-topbar-height) 1fr var(--app-statusbar-height)" }}
        >
          <Topbar />

          <div className="grid min-h-0 min-w-0" style={{ gridTemplateColumns: contentColumns }}>
            {/* Sidebar */}
            {!sidebarCollapsed && <Sidebar width={effectiveSidebarWidth} />}

            {/* Sidebar resize handle */}
            {!sidebarCollapsed && (
              <div
                className="group relative z-10 w-[3px] shrink-0 cursor-col-resize bg-transparent transition-colors hover:bg-[var(--border-subtle)] active:bg-primary"
                onMouseDown={handleSidebarResizeStart}
              >
                <div className="absolute inset-y-0 -left-1 -right-1" />
              </div>
            )}

            {/* Main content */}
            <main className="flex min-h-0 min-w-0 flex-col overflow-hidden bg-[var(--surface-editor)]">
              <div className="flex h-full flex-col">
                {hasTabs && <WorkspaceTabBar />}
                <WorkspaceContent />
              </div>
            </main>

            {/* Agent resize handle */}
            {agentOpen && (
              <div
                className="group relative z-10 w-[3px] shrink-0 cursor-col-resize bg-transparent transition-colors hover:bg-[var(--border-subtle)] active:bg-primary"
                onMouseDown={handleAgentResizeStart}
              >
                <div className="absolute inset-y-0 -left-1 -right-1" />
              </div>
            )}

            {/* Agent panel */}
            {agentOpen && (
              <AgentPanel open={agentOpen} onClose={() => setAgentOpen(false)} width={agentWidth} />
            )}
          </div>

          <StatusBar />
        </div>
      </div>
      <CommandPalette />
      <QuickOpen />
      <ConnectionDialog />
      <ActionConfirmationHost />
    </>
  );
}
