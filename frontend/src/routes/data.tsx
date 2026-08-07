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
      useWorkspaceStore.getState().openDbObject(
        createDbObjectTab(connectionId, schema, table, "table", "data", false),
      );
    }
    navigate({ to: "/" });
  }, [schema, table, navigate]);
  return null;
}
