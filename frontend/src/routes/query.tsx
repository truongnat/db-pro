import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useEffect } from "react";

import { useConnectionStore } from "@/commons/stores/connection.store";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { createQueryTabFromExplorerContext } from "@/modules/query/controllers/query-workspace.controller";

export const Route = createFileRoute("/query")({
  component: QueryRedirect,
});

function QueryRedirect() {
  const navigate = useNavigate();
  useEffect(() => {
    const connectionId = useConnectionStore.getState().explorerConnectionId;
    if (connectionId) {
      const { tabs, openTab, activateTab } = useWorkspaceStore.getState();
      const existing = tabs.find((t) => t.kind === "query" && t.connectionId === connectionId);
      if (!existing) {
        const tab = createQueryTabFromExplorerContext(connectionId);
        if (tab) openTab(tab);
      } else {
        activateTab(existing.id);
      }
    }
    navigate({ to: "/" });
  }, [navigate]);
  return null;
}
