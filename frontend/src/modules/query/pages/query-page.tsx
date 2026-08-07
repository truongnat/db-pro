import { useCallback, useEffect, useRef, useState } from "react";

import { useConnectionStore } from "@/commons/stores/connection.store";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { useTranslation } from "@/commons/locales/useTranslation";
import { createQueryTab } from "@/commons/factories/tab-factories";
import { useConnectionList } from "@/modules/connection/queries/connection.queries";

import { QueryTabContent } from "../components/query-tab-content";
import { RunConfigDialog } from "../components/run-config-dialog";
import { RunConfigList } from "../components/run-config-list";
import { SavedQueriesTree } from "../components/saved-queries-tree";
import { SqlFileOperations } from "../components/sql-file-operations";
import { createExplorerQueryContext, setTabSql } from "../controllers/query-workspace.controller";
import type { RunConfig } from "../types/query.types";

export function QueryPage() {
  const { t } = useTranslation();
  const explorerConnectionId = useConnectionStore((s) => s.explorerConnectionId);
  const { data: connections } = useConnectionList();

  const activeTab = useWorkspaceStore((s) => {
    const tab = s.tabs.find((t) => t.id === s.activeTabId);
    return tab?.kind === "query" ? tab : undefined;
  });
  const activeTabId = useWorkspaceStore((s) => s.activeTabId);

  const tabConnectionId = activeTab?.connectionId ?? null;
  const sql = activeTab?.data.sql ?? "";

  const [runConfigOpen, setRunConfigOpen] = useState(false);

  const initializedRef = useRef<string | null>(null);
  useEffect(() => {
    if (!explorerConnectionId) return;
    if (initializedRef.current === explorerConnectionId) return;
    initializedRef.current = explorerConnectionId;

    const { tabs } = useWorkspaceStore.getState();
    const hasTabsForConnection = tabs.some(
      (t) => t.kind === "query" && t.connectionId === explorerConnectionId,
    );
    if (!hasTabsForConnection) {
      const tab = createQueryTab(explorerConnectionId, {
        context: createExplorerQueryContext(connections ?? [], explorerConnectionId),
      });
      useWorkspaceStore.getState().openTab(tab);
    }
  }, [explorerConnectionId, connections]);

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

  if (!tabConnectionId) {
    return (
      <div className="flex items-center justify-center py-12">
        <p className="text-muted-foreground">{t("query.connectFirst")}</p>
      </div>
    );
  }

  return (
    <div className="flex h-full">
      <div
        className="flex flex-col border-r border-border"
        style={{ width: "220px", minWidth: "180px" }}
      >
        <div className="border-b border-border p-2">
          <SqlFileOperations sql={sql} onSqlLoaded={(s) => activeTabId && setTabSql(activeTabId, s)} />
        </div>
        <div className="flex-1 overflow-auto">
          <SavedQueriesTree
            connectionId={tabConnectionId}
            onSelectQuery={handleSelectSavedQuery}
          />
        </div>
        <div className="border-t border-border" style={{ maxHeight: "40%" }}>
          <RunConfigList
            connectionId={tabConnectionId}
            onSelect={handleSelectRunConfig}
            onNew={() => setRunConfigOpen(true)}
          />
        </div>
      </div>

      {activeTabId ? (
        <QueryTabContent tabId={activeTabId} onOpenRunConfig={() => setRunConfigOpen(true)} />
      ) : (
        <div className="flex flex-1 items-center justify-center">
          <p className="text-muted-foreground">{t("query.connectFirst")}</p>
        </div>
      )}

      <RunConfigDialog
        open={runConfigOpen}
        onClose={() => setRunConfigOpen(false)}
        connectionId={tabConnectionId}
        defaultSql={sql.trim()}
      />
    </div>
  );
}
