import { CommandPalette } from "@/commons/components/command-palette";
import { WorkspaceContent } from "@/commons/components/workspace-content";
import { WorkspaceTabBar } from "@/commons/components/workspace-tab-bar";
import { useCommandPalette } from "@/commons/hooks/use-command-palette";
import { useShellStore } from "@/commons/stores/shell.store";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { ConnectionDialog } from "@/modules/connection/components/connection-dialog";

import { ActivityBar } from "./activity-bar";
import { Sidebar } from "./sidebar";
import { StatusBar } from "./status-bar";
import { Topbar } from "./topbar";

export function AppShell() {
  const sidebarCollapsed = useShellStore((s) => s.sidebarCollapsed);
  const hasTabs = useWorkspaceStore((s) => s.tabs.length > 0);
  useCommandPalette();

  return (
    <>
    <div className="grid h-screen overflow-hidden" style={{ gridTemplateColumns: "var(--app-activity-bar-width) 1fr" }}>
      <ActivityBar />

      <div className="grid min-h-0 min-w-0" style={{ gridTemplateRows: "var(--app-topbar-height) 1fr var(--app-statusbar-height)" }}>
        <Topbar />

        <div
          className="grid min-h-0 min-w-0 transition-[grid-template-columns] duration-200 ease-out"
          style={{
            gridTemplateColumns: sidebarCollapsed
              ? "var(--app-sidebar-collapsed-width) 1fr"
              : "var(--app-sidebar-width) 1fr",
          }}
        >
          <Sidebar />

          <main className="flex min-h-0 min-w-0 flex-col overflow-hidden bg-background">
            <div className="flex h-full flex-col">
              {hasTabs && <WorkspaceTabBar />}
              <WorkspaceContent />
            </div>
          </main>
        </div>

        <StatusBar />
      </div>
    </div>
    <CommandPalette />
    <ConnectionDialog />
    </>
  );
}
