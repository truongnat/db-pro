import { useCallback } from "react";

import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { createDbObjectTab } from "@/commons/factories/tab-factories";
import type { DbObjectTabData } from "@/commons/types/workspace.types";

export function useSidebarTabOps() {
  const openTab = useWorkspaceStore((s) => s.openTab);
  const promotePreview = useWorkspaceStore((s) => s.promotePreview);

  const openSchemaPreview = useCallback(
    (
      connectionId: string,
      schema: string,
      objectName: string,
      objectType: DbObjectTabData["objectType"],
    ) => {
      const tab = createDbObjectTab(connectionId, schema, objectName, objectType, "structure", true);
      openTab(tab);
    },
    [openTab],
  );

  const promoteSchemaPreview = useCallback(
    (
      connectionId: string,
      schema: string,
      objectName: string,
    ) => {
      const resourceKey = `dbobj:${schema}.${objectName}:${connectionId}`;
      const tab = useWorkspaceStore.getState().tabs.find((t) => t.resourceKey === resourceKey);
      if (tab) promotePreview(tab.id);
    },
    [promotePreview],
  );

  const openTableData = useCallback(
    (connectionId: string, schema: string, table: string) => {
      const tab = createDbObjectTab(connectionId, schema, table, "table", "data", false);
      openTab(tab);
    },
    [openTab],
  );

  return { openSchemaPreview, promoteSchemaPreview, openTableData };
}
