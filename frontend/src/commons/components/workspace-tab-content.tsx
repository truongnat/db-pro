import { useEffect } from "react";

import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { useQueryModuleStore } from "@/modules/query/state/query.store";

interface WorkspaceTabContentProps {
  children: React.ReactNode;
}

export function WorkspaceTabContent({ children }: WorkspaceTabContentProps) {
  const activeTabId = useWorkspaceStore((s) => s.activeTabId);
  const activeTab = useWorkspaceStore((s) => s.tabs.find((t) => t.id === s.activeTabId));

  useEffect(() => {
    if (!activeTab || activeTab.kind !== "query") return;

    const queryStore = useQueryModuleStore.getState();
    const existingTab = queryStore.tabs.find((t) => t.id === activeTab.id);

    if (!existingTab) {
      useQueryModuleStore.setState({
        tabs: [
          ...queryStore.tabs,
          {
            id: activeTab.id,
            title: activeTab.title,
            sql: activeTab.data.sql,
            status: activeTab.data.status,
            error: activeTab.data.error,
            result: activeTab.data.result,
            explainPlan: activeTab.data.explainPlan,
            sort: activeTab.data.sort,
            multiResults: activeTab.data.multiResults,
            multiResultIndex: activeTab.data.multiResultIndex,
          },
        ],
        activeTabId: activeTab.id,
      });
    } else {
      useQueryModuleStore.setState({
        tabs: queryStore.tabs.map((t) =>
          t.id === activeTab.id
            ? {
                ...t,
                sql: activeTab.data.sql,
                status: activeTab.data.status,
                error: activeTab.data.error,
                result: activeTab.data.result,
                explainPlan: activeTab.data.explainPlan,
                sort: activeTab.data.sort,
                multiResults: activeTab.data.multiResults,
                multiResultIndex: activeTab.data.multiResultIndex,
              }
            : t,
        ),
        activeTabId: activeTab.id,
      });
    }
  }, [activeTabId, activeTab]);

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
