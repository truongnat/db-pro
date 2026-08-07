import { useCallback } from "react";

import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { createDbObjectTab } from "@/commons/factories/tab-factories";
import type { DbObjectTabData } from "@/commons/types/workspace.types";

export function useSidebarTabOps() {
  const openDbObject = useWorkspaceStore((s) => s.openDbObject);
  const promotePreview = useWorkspaceStore((s) => s.promotePreview);

  const openSchemaPreview = useCallback(
    (
      connectionId: string,
      schema: string,
      objectName: string,
      objectType: DbObjectTabData["objectType"],
    ) => {
      const tab = createDbObjectTab(connectionId, schema, objectName, objectType, "columns", true);
      openDbObject(tab);
    },
    [openDbObject],
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
    (
      connectionId: string,
      schema: string,
      objectName: string,
      objectType: DbObjectTabData["objectType"] = "table",
    ) => {
      const resourceKey = `dbobj:${schema}.${objectName}:${connectionId}`;
      const existing = useWorkspaceStore.getState().tabs.find((t) => t.resourceKey === resourceKey);

      if (existing && existing.kind === "db-object") {
        if (existing.preview) {
          useWorkspaceStore.getState().promotePreview(existing.id);
        }
        useWorkspaceStore.getState().setDbObjectSection(existing.id, "data");
        useWorkspaceStore.getState().activateTab(existing.id);
        return;
      }

      const tab = createDbObjectTab(connectionId, schema, objectName, objectType, "data", false);
      openDbObject(tab);
    },
    [openDbObject],
  );

  return { openSchemaPreview, promoteSchemaPreview, openTableData };
}
