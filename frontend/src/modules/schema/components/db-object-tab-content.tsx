import { useState } from "react";

import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { useTranslation } from "@/commons/locales/useTranslation";
import { createQueryTabForObject } from "@/modules/query/controllers/query-workspace.controller";

import { useTableInfo, useTableDdl } from "@/modules/schema/queries/schema.queries";
import { ColumnList } from "@/modules/schema/components/column-list";
import { DdlViewer } from "@/modules/schema/components/ddl-viewer";
import { DdlEditor } from "@/modules/schema/components/ddl-editor/ddl-editor";
import { ForeignKeyList } from "@/modules/schema/components/foreign-key-list";
import { GenerateCrud } from "@/modules/schema/components/generate-crud";
import { IndexManager } from "@/modules/schema/components/index-manager";
import { TriggerManager } from "@/modules/schema/components/trigger-manager";

import { DbObjectWorkspace } from "./db-object-workspace";
import { ObjectSectionLayout } from "./object-section-layout";
import { ObjectContextHeader } from "./object-context-header";
import { DataSection } from "./data/data-section";

interface DbObjectTabContentProps {
  tabId: string;
  connectionId: string | null;
  schema: string;
  objectName: string;
  objectType: "table" | "view" | "function" | "sequence" | "type";
}

type ToolbarAction = "ddlEditor" | "generateCrud";

export function DbObjectTabContent({
  tabId,
  connectionId,
  schema,
  objectName,
  objectType,
}: DbObjectTabContentProps) {
  const { t } = useTranslation();
  const [toolbarAction, setToolbarAction] = useState<ToolbarAction | null>(null);

  const activeSection = useWorkspaceStore((s) => {
    const tab = s.tabs.find((t) => t.id === tabId);
    return tab?.kind === "db-object" ? tab.data.activeSection : "columns";
  });
  const setSection = useWorkspaceStore((s) => s.setDbObjectSection);

  const isTableOrView = objectType === "table" || objectType === "view";

  const tableInfo = useTableInfo(connectionId, schema, objectName);
  const tableDdl = useTableDdl(connectionId, schema, objectName, activeSection === "ddl" || toolbarAction === "ddlEditor");

  if (!connectionId) return null;

  const openQueryTab = (sql: string, title: string) => {
    const tab = createQueryTabForObject(connectionId, schema, { title, sql });
    useWorkspaceStore.getState().openTab(tab);
  };

  const handleRefresh = () => {
    tableInfo.refetch();
    if (activeSection === "ddl" || toolbarAction === "ddlEditor") {
      tableDdl.refetch();
    }
  };

  const handleOpenSelect = () => {
    const qualified = `"${schema.replace(/"/g, '""')}"."${objectName.replace(/"/g, '""')}"`;
    openQueryTab(`SELECT * FROM ${qualified} LIMIT 100;`, `SELECT ${objectName}`);
  };

  const handleOpenDdl = () => {
    if (tableDdl.data) {
      openQueryTab(tableDdl.data, `DDL ${objectName}`);
    } else {
      tableDdl.refetch().then((res) => {
        if (res.data) openQueryTab(res.data, `DDL ${objectName}`);
      });
    }
  };

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
      <ObjectContextHeader
        connectionId={connectionId}
        schema={schema}
        objectName={objectName}
        objectType={objectType}
        onRefresh={handleRefresh}
        onOpenSelect={handleOpenSelect}
        onOpenDdl={handleOpenDdl}
        onOpenDdlEditor={() => setToolbarAction("ddlEditor")}
        onGenerateSql={() => setToolbarAction("generateCrud")}
      />
      <DbObjectWorkspace
        activeSection={activeSection}
        onSelectSection={(section) => {
          setToolbarAction(null);
          setSection(tabId, section);
        }}
      >
        {toolbarAction === "ddlEditor" && isTableOrView && (
          <ObjectSectionLayout>
            <DdlEditor connectionId={connectionId} schema={schema} table={objectName} />
          </ObjectSectionLayout>
        )}
        {toolbarAction === "generateCrud" && isTableOrView && tableInfo.data && (
          <ObjectSectionLayout>
            <GenerateCrud schema={schema} table={objectName} columns={tableInfo.data.columns} />
          </ObjectSectionLayout>
        )}
        {!toolbarAction && (
          <>
            {activeSection === "data" && isTableOrView && (
              <DataSection tabId={tabId} connectionId={connectionId} schema={schema} table={objectName} />
            )}
            {activeSection === "columns" && isTableOrView && tableInfo.data && (
              <ObjectSectionLayout>
                <ColumnList columns={tableInfo.data.columns} />
              </ObjectSectionLayout>
            )}
            {activeSection === "indexes" && isTableOrView && tableInfo.data && (
              <ObjectSectionLayout>
                <IndexManager
                  connectionId={connectionId}
                  schema={schema}
                  table={objectName}
                  columns={tableInfo.data.columns}
                  indexes={tableInfo.data.indexes}
                />
              </ObjectSectionLayout>
            )}
            {activeSection === "relations" && isTableOrView && tableInfo.data && (
              <ObjectSectionLayout>
                <ForeignKeyList foreignKeys={tableInfo.data.foreignKeys} />
              </ObjectSectionLayout>
            )}
            {activeSection === "triggers" && isTableOrView && (
              <ObjectSectionLayout>
                <TriggerManager connectionId={connectionId} schema={schema} table={objectName} />
              </ObjectSectionLayout>
            )}
            {activeSection === "ddl" && isTableOrView && (
              <ObjectSectionLayout>
                <DdlViewer
                  ddl={tableDdl.data ?? null}
                  isLoading={tableDdl.isLoading}
                  error={
                    tableDdl.isError
                      ? (tableDdl.error as { userMessage?: string })?.userMessage ?? t("common.states.error")
                      : null
                  }
                />
              </ObjectSectionLayout>
            )}
            {!isTableOrView && (
              <div className="flex h-full items-center justify-center p-8">
                <p className="text-sm text-muted-foreground">
                  {t("dbObject.unsupportedObjectType", { type: objectType })}
                </p>
              </div>
            )}
            {tableInfo.isLoading && (activeSection === "columns" || activeSection === "indexes" || activeSection === "relations") && (
              <div className="flex h-full min-h-0 items-center justify-center">
                <div className="p-4 text-sm text-muted-foreground">
                  {t("common.states.loading")}
                </div>
              </div>
            )}
            {tableInfo.isError && (activeSection === "columns" || activeSection === "indexes" || activeSection === "relations") && (
              <div className="flex h-full min-h-0 items-center justify-center">
                <div className="p-4 text-sm text-destructive">
                  {(tableInfo.error as { userMessage?: string })?.userMessage ?? t("common.states.error")}
                </div>
              </div>
            )}
          </>
        )}
      </DbObjectWorkspace>
    </div>
  );
}
