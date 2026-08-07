import { useTranslation } from "@/commons/locales/useTranslation";
import { useConnectionValid } from "@/commons/hooks/use-connection-valid";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";

import { WelcomeView } from "./welcome-view";
import { QueryTabContent } from "@/modules/query/components/query-tab-content";
import { DbObjectTabContent } from "@/modules/schema/components/db-object-tab-content";

function OrphanedTabView({ tabTitle }: { tabTitle: string }) {
  const { t } = useTranslation();
  return (
    <div className="flex h-full items-center justify-center p-8">
      <div className="max-w-sm text-center">
        <p className="text-sm font-medium text-foreground">{tabTitle}</p>
        <p className="mt-1 text-xs text-[var(--app-text-muted)]">{t("workspace.connectionUnavailable")}</p>
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
    return <OrphanedTabView tabTitle={activeTab.title} />;
  }

  switch (activeTab.kind) {
    case "query":
      return (
        <QueryTabContent
          tabId={activeTab.id}
          onOpenRunConfig={() => {}}
        />
      );
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
    default:
      return <WelcomeView />;
  }
}
