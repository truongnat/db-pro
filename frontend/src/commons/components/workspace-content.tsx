import { useTranslation } from "@/commons/locales/useTranslation";
import { useConnectionValid } from "@/commons/hooks/use-connection-valid";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { useConnectionList } from "@/modules/connection/queries/connection.queries";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { ChevronDown } from "lucide-react";

import { WelcomeView } from "./welcome-view";
import { QueryTabContent } from "@/modules/query/components/query-tab-content";
import { DbObjectTabContent } from "@/modules/schema/components/db-object-tab-content";
import { SchemaWorkspaceContent } from "@/modules/schema/components/schema-workspace-content";

function OrphanedTabView({ tabId, tabTitle }: { tabId: string; tabTitle: string }) {
  const { t } = useTranslation();
  const connections = useConnectionList();

  const handleChangeConnection = (newConnId: string) => {
    useWorkspaceStore.getState().reassignTabConnection(tabId, newConnId);
  };

  const handleCloseTab = () => {
    useWorkspaceStore.getState().closeTab(tabId);
  };

  const availableConnections = connections.data ?? [];

  return (
    <div className="flex h-full items-center justify-center p-8">
      <div className="max-w-sm space-y-3 text-center">
        <p className="text-sm font-medium text-foreground">{tabTitle}</p>
        <p className="text-xs text-[var(--text-secondary)]">
          {t("workspace.connectionUnavailable")}
        </p>
        <div className="flex flex-col gap-2">
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                type="button"
                size="sm"
                variant="outline"
                className="flex items-center gap-1 text-xs"
              >
                {t("workspace.changeConnection")}
                <ChevronDown className="h-3 w-3" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="center">
              {availableConnections.map((conn) => (
                <DropdownMenuItem key={conn.id} onClick={() => handleChangeConnection(conn.id)}>
                  {conn.name}
                </DropdownMenuItem>
              ))}
              {availableConnections.length === 0 && (
                <DropdownMenuItem disabled>{t("common.states.empty")}</DropdownMenuItem>
              )}
            </DropdownMenuContent>
          </DropdownMenu>
          <Button
            type="button"
            size="sm"
            variant="ghost"
            className="text-xs text-[var(--text-secondary)]"
            onClick={handleCloseTab}
          >
            {t("common.actions.close")}
          </Button>
        </div>
      </div>
    </div>
  );
}

export function WorkspaceContent() {
  const activeTab = useWorkspaceStore((s) => {
    if (!s.activeTabId) return null;
    return s.tabs.find((t) => t.id === s.activeTabId) ?? null;
  });
  const tabCount = useWorkspaceStore((s) => s.tabs.length);
  const connectionValid = useConnectionValid(activeTab?.connectionId ?? null);

  if (tabCount === 0) {
    return <WelcomeView />;
  }

  if (!activeTab) {
    return <WelcomeView />;
  }

  if (!connectionValid) {
    return <OrphanedTabView tabId={activeTab.id} tabTitle={activeTab.title} />;
  }

  switch (activeTab.kind) {
    case "query":
      return <QueryTabContent tabId={activeTab.id} />;
    case "db-object":
      return (
        <DbObjectTabContent
          key={activeTab.resourceKey}
          tabId={activeTab.id}
          connectionId={activeTab.connectionId}
          schema={activeTab.data.schema}
          objectName={activeTab.data.objectName}
          objectType={activeTab.data.objectType}
        />
      );
    case "schema-workspace":
      return (
        <SchemaWorkspaceContent
          key={activeTab.resourceKey}
          tabId={activeTab.id}
          connectionId={activeTab.connectionId!}
          schema={activeTab.data.schema}
        />
      );
    default:
      return <WelcomeView />;
  }
}
