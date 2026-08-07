import { useEffect, useRef } from "react";

import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import type { QueryTabData } from "@/commons/types/workspace.types";
import { useQueryModuleStore } from "@/modules/query/state/query.store";
import type { QueryTab } from "@/modules/query/state/query.store";

interface WorkspaceTabContentProps {
  children: React.ReactNode;
}

function workspaceDataToLegacy(data: QueryTabData): Omit<QueryTab, "id" | "title"> {
  return {
    sql: data.sql,
    status: data.status,
    error: data.error,
    result: data.result,
    explainPlan: data.explainPlan,
    sort: data.sort,
    multiResults: data.multiResults,
    multiResultIndex: data.multiResultIndex,
  };
}

function legacyTabToWorkspaceData(tab: QueryTab): QueryTabData {
  return {
    sql: tab.sql,
    status: tab.status,
    error: tab.error,
    result: tab.result,
    explainPlan: tab.explainPlan,
    sort: tab.sort,
    multiResults: tab.multiResults,
    multiResultIndex: tab.multiResultIndex,
  };
}

function legacyDataEqualsWorkspaceData(legacy: QueryTab, workspace: QueryTabData): boolean {
  return (
    legacy.sql === workspace.sql &&
    legacy.status === workspace.status &&
    legacy.error === workspace.error &&
    legacy.result === workspace.result &&
    legacy.explainPlan === workspace.explainPlan &&
    legacy.sort.column === workspace.sort.column &&
    legacy.sort.direction === workspace.sort.direction &&
    legacy.multiResults === workspace.multiResults &&
    legacy.multiResultIndex === workspace.multiResultIndex
  );
}

export function WorkspaceTabContent({ children }: WorkspaceTabContentProps) {
  const activeTabId = useWorkspaceStore((s) => s.activeTabId);
  const activeTab = useWorkspaceStore((s) => s.tabs.find((t) => t.id === s.activeTabId));
  const suppressNextLegacySync = useRef(false);

  // workspace → legacy: fires ONLY when active tab ID changes (tab switch)
  useEffect(() => {
    if (!activeTabId) return;
    const workspaceTab = useWorkspaceStore.getState().tabs.find((t) => t.id === activeTabId);
    if (!workspaceTab || workspaceTab.kind !== "query") return;

    const queryStore = useQueryModuleStore.getState();
    const existingTab = queryStore.tabs.find((t) => t.id === workspaceTab.id);
    const legacyData = workspaceDataToLegacy(workspaceTab.data);

    suppressNextLegacySync.current = true;

    if (!existingTab) {
      useQueryModuleStore.setState({
        tabs: [...queryStore.tabs, { id: workspaceTab.id, title: workspaceTab.title, ...legacyData }],
        activeTabId: workspaceTab.id,
      });
    } else {
      useQueryModuleStore.setState({
        tabs: queryStore.tabs.map((t) => (t.id === workspaceTab.id ? { ...t, ...legacyData } : t)),
        activeTabId: workspaceTab.id,
      });
    }
  }, [activeTabId]);

  // legacy → workspace: subscribes to legacy store, writes back on every change
  useEffect(() => {
    const unsub = useQueryModuleStore.subscribe((state) => {
      if (suppressNextLegacySync.current) {
        suppressNextLegacySync.current = false;
        return;
      }

      if (!activeTabId) return;
      const legacyTab = state.tabs.find((t) => t.id === activeTabId);
      if (!legacyTab) return;

      const workspaceState = useWorkspaceStore.getState();
      const workspaceTab = workspaceState.tabs.find((t) => t.id === activeTabId);
      if (!workspaceTab || workspaceTab.kind !== "query") return;

      if (!legacyDataEqualsWorkspaceData(legacyTab, workspaceTab.data)) {
        workspaceState.updateTabData(activeTabId, () => legacyTabToWorkspaceData(legacyTab));
        if (state.activeTabId !== activeTabId) {
          return;
        }
      }
    });
    return unsub;
  }, [activeTabId]);

  if (!activeTab) {
    return (
      <div className="flex items-center justify-center py-12">
        <p className="text-muted-foreground">No active tab</p>
      </div>
    );
  }

  if (activeTab.kind !== "query") {
    return (
      <div className="flex items-center justify-center py-12">
        <p className="text-muted-foreground">Tab type not yet supported</p>
      </div>
    );
  }

  return <>{children}</>;
}
