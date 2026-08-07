import { useCallback } from "react";

import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import {
  createSchemaObjectTab,
  createTableDataTab,
} from "@/commons/factories/tab-factories";
import type { SchemaObjectTabData } from "@/commons/types/workspace.types";

export function useSidebarTabOps() {
  const openTab = useWorkspaceStore((s) => s.openTab);
  const promotePreview = useWorkspaceStore((s) => s.promotePreview);

  const openSchemaPreview = useCallback(
    (
      connectionId: string,
      schema: string,
      objectName: string,
      objectType: SchemaObjectTabData["objectType"],
    ) => {
      const tab = createSchemaObjectTab(connectionId, schema, objectName, objectType);
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
      const resourceKey = `object:${schema}.${objectName}:${connectionId}`;
      const tab = useWorkspaceStore.getState().tabs.find((t) => t.resourceKey === resourceKey);
      if (tab) promotePreview(tab.id);
    },
    [promotePreview],
  );

  const openTableData = useCallback(
    (connectionId: string, schema: string, table: string) => {
      const tab = createTableDataTab(connectionId, schema, table);
      openTab(tab);
    },
    [openTab],
  );

  return { openSchemaPreview, promoteSchemaPreview, openTableData };
}
