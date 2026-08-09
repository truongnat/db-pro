import { useCallback } from "react";

import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { useRecentStore } from "@/commons/stores/recent.store";
import { createDbObjectTab } from "@/commons/factories/tab-factories";
import type { DbObjectTabData } from "@/commons/types/workspace.types";

function recordRecentResource(connectionId: string, schema: string, objectName: string) {
  useRecentStore.getState().addRecentResource({
    resourceKey: `dbobj:${schema}.${objectName}:${connectionId}`,
    kind: "db-object",
    connectionId,
    schema,
    objectName,
  });
}

export function useSidebarTabOps() {
  const openDbObject = useWorkspaceStore((s) => s.openDbObject);
  const promotePreview = useWorkspaceStore((s) => s.promotePreview);

  /**
   * Single-click object preview.
   *
   * Data is the primary user intent when opening a table/view from Explorer,
   * so previews open directly on the Data section instead of configuration
   * metadata (Columns). The preview behavior is preserved so navigating the
   * tree does not create a permanent tab for every click.
   */
  const openSchemaPreview = useCallback(
    (
      connectionId: string,
      schema: string,
      objectName: string,
      objectType: DbObjectTabData["objectType"],
    ) => {
      const tab = createDbObjectTab(connectionId, schema, objectName, objectType, "data", true);
      openDbObject(tab);
      recordRecentResource(connectionId, schema, objectName);
    },
    [openDbObject],
  );

  const promoteSchemaPreview = useCallback(
    (connectionId: string, schema: string, objectName: string) => {
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
      recordRecentResource(connectionId, schema, objectName);
    },
    [openDbObject],
  );

  /**
   * Open a DB Object tab with the Columns/Structure section active.
   * If a tab already exists for this resource it is reused and activated.
   */
  const openObjectStructure = useCallback(
    (
      connectionId: string,
      schema: string,
      objectName: string,
      objectType: DbObjectTabData["objectType"],
    ) => {
      const resourceKey = `dbobj:${schema}.${objectName}:${connectionId}`;
      const existing = useWorkspaceStore.getState().tabs.find((t) => t.resourceKey === resourceKey);

      if (existing && existing.kind === "db-object") {
        if (existing.preview) {
          useWorkspaceStore.getState().promotePreview(existing.id);
        }
        useWorkspaceStore.getState().setDbObjectSection(existing.id, "columns");
        useWorkspaceStore.getState().activateTab(existing.id);
        return;
      }

      const tab = createDbObjectTab(connectionId, schema, objectName, objectType, "columns", false);
      openDbObject(tab);
      recordRecentResource(connectionId, schema, objectName);
    },
    [openDbObject],
  );

  return { openSchemaPreview, promoteSchemaPreview, openTableData, openObjectStructure };
}
