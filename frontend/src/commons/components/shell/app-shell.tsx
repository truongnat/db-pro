import { useCallback, useRef } from "react";

import { CommandPalette } from "@/commons/components/command-palette";
import { QuickOpen } from "@/commons/components/quick-open";
import { WorkspaceContent } from "@/commons/components/workspace-content";
import { WorkspaceTabBar } from "@/commons/components/workspace-tab-bar";
import { useCommandPalette } from "@/commons/hooks/use-command-palette";
import { useQuickOpen } from "@/commons/hooks/use-quick-open";
import { useShellStore } from "@/commons/stores/shell.store";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { ConnectionDialog } from "@/modules/connection/components/connection-dialog";

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
  const hasTabs = useWorkspaceStore((s) => s.tabs.length > 0);
  const draggingRef = useRef(false);
  useCommandPalette();
  useQuickOpen();

  const handleResizeStart = useCallback(
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

  const effectiveSidebarWidth = sidebarCollapsed ? 0 : sidebarWidth;

  return (
    <>
    <div className="grid h-screen overflow-hidden" style={{ gridTemplateColumns: "var(--app-activity-bar-width) 1fr" }}>
      <ActivityBar />

      <div className="grid min-h-0 min-w-0" style={{ gridTemplateRows: "var(--app-topbar-height) 1fr var(--app-statusbar-height)" }}>
        <Topbar />

        <div
          className="grid min-h-0 min-w-0"
          style={{
            gridTemplateColumns: sidebarCollapsed
              ? "0px 1fr"
              : `${effectiveSidebarWidth}px 3px 1fr`,
          }}
        >
          <Sidebar width={effectiveSidebarWidth} />

          {/* Resize handle */}
          {!sidebarCollapsed && (
            <div
              className="group relative z-10 w-[3px] shrink-0 cursor-col-resize bg-transparent transition-colors hover:bg-[var(--app-border-subtle)] active:bg-primary"
              onMouseDown={handleResizeStart}
            >
              <div className="absolute inset-y-0 -left-1 -right-1" />
            </div>
          )}

          <main className="flex min-h-0 min-w-0 flex-col overflow-hidden bg-background">
            <div className="flex h-full flex-col">
              {hasTabs && <WorkspaceTabBar />}
              <WorkspaceContent />
            </div>
          </main>

          <AgentPanel
            open={agentOpen}
            onClose={() => setAgentOpen(false)}
          />
        </div>

        <StatusBar />
      </div>
    </div>
    <CommandPalette />
    <QuickOpen />
    <ConnectionDialog />
    </>
  );
}
