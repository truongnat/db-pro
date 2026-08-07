import { useCallback } from "react";

import { useConnectionStore } from "@/commons/stores/connection.store";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { setTabSql } from "@/modules/query/controllers/query-workspace.controller";
import { SavedQueriesTree } from "@/modules/query/components/saved-queries-tree";
import { RunConfigList } from "@/modules/query/components/run-config-list";
import { SqlFileOperations } from "@/modules/query/components/sql-file-operations";
import type { RunConfig } from "@/modules/query/types/query.types";

export function QuerySavedView() {
  const explorerConnectionId = useConnectionStore((s) => s.explorerConnectionId);
  const activeTab = useWorkspaceStore((s) => {
    const tab = s.tabs.find((t) => t.id === s.activeTabId);
    return tab?.kind === "query" ? tab : undefined;
  });
  const activeTabId = useWorkspaceStore((s) => s.activeTabId);
  const sql = activeTab?.data.sql ?? "";
  const tabConnectionId = activeTab?.connectionId ?? explorerConnectionId;

  const handleSelectSavedQuery = useCallback(
    (querySql: string) => {
      if (!activeTabId) return;
      setTabSql(activeTabId, querySql);
    },
    [activeTabId],
  );

  const handleSelectRunConfig = useCallback(
    (config: RunConfig) => {
      if (!activeTabId) return;
      setTabSql(activeTabId, config.sql);
    },
    [activeTabId],
  );

  if (!tabConnectionId) return null;

  return (
    <div className="flex min-h-0 flex-col">
      <div className="border-b border-[var(--app-border-subtle)] p-2">
        <SqlFileOperations sql={sql} onSqlLoaded={(s) => activeTabId && setTabSql(activeTabId, s)} />
      </div>
      <div className="flex-1 overflow-auto">
        <SavedQueriesTree
          connectionId={tabConnectionId}
          onSelectQuery={handleSelectSavedQuery}
        />
      </div>
      <div className="border-t border-[var(--app-border-subtle)]" style={{ maxHeight: "40%" }}>
        <RunConfigList
          connectionId={tabConnectionId}
          onSelect={handleSelectRunConfig}
          onNew={() => {}}
        />
      </div>
    </div>
  );
}
