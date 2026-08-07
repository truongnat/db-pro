import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useEffect } from "react";

import { createQueryTab } from "@/commons/factories/tab-factories";
import { useConnectionStore } from "@/commons/stores/connection.store";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";

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
        openTab(createQueryTab(connectionId));
      } else {
        activateTab(existing.id);
      }
    }
    navigate({ to: "/" });
  }, [navigate]);
  return null;
}
