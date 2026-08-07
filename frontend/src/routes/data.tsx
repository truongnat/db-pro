import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useEffect } from "react";

import { createDbObjectTab } from "@/commons/factories/tab-factories";
import { useConnectionStore } from "@/commons/stores/connection.store";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";

export const Route = createFileRoute("/data")({
  validateSearch: (search) => ({
    schema: (search.schema as string) ?? "",
    table: (search.table as string) ?? "",
  }),
  component: DataRedirect,
});

function DataRedirect() {
  const navigate = useNavigate();
  const { schema, table } = Route.useSearch();
  useEffect(() => {
    const connectionId = useConnectionStore.getState().explorerConnectionId;
    if (connectionId && schema && table) {
      const resourceKey = `dbobj:${schema}.${table}:${connectionId}`;
      const existing = useWorkspaceStore.getState().tabs.find((t) => t.resourceKey === resourceKey);
      if (existing && existing.kind === "db-object") {
        if (existing.preview) useWorkspaceStore.getState().promotePreview(existing.id);
        useWorkspaceStore.getState().setDbObjectSection(existing.id, "data");
        useWorkspaceStore.getState().activateTab(existing.id);
      } else {
        useWorkspaceStore.getState().openDbObject(
          createDbObjectTab(connectionId, schema, table, "table", "data", false),
        );
      }
    }
    navigate({ to: "/" });
  }, [schema, table, navigate]);
  return null;
}
