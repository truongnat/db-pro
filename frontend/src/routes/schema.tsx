import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useEffect } from "react";

import { createSchemaObjectTab } from "@/commons/factories/tab-factories";
import { useConnectionStore } from "@/commons/stores/connection.store";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import type { SchemaObjectTabData } from "@/commons/types/workspace.types";

export const Route = createFileRoute("/schema")({
  validateSearch: (search) => ({
    schema: (search.schema as string) ?? "",
    object: (search.object as string) ?? "",
    type: ((search.type as string) ?? "table") as SchemaObjectTabData["objectType"],
  }),
  component: SchemaRedirect,
});

function SchemaRedirect() {
  const navigate = useNavigate();
  const { schema, object, type } = Route.useSearch();
  useEffect(() => {
    const connectionId = useConnectionStore.getState().activeConnectionId;
    if (connectionId && schema && object) {
      useWorkspaceStore.getState().openTab(createSchemaObjectTab(connectionId, schema, object, type));
    }
    navigate({ to: "/" });
  }, [schema, object, type, navigate]);
  return null;
}
