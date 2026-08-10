import { useCallback, useState } from "react";

import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { useTranslation } from "@/commons/locales/useTranslation";
import { createQueryTabForObject } from "@/modules/query/controllers/query-workspace.controller";
import { getDialectForConnection } from "@/modules/query/sql/dialect";

import { useTableInfo, useTableDdl, useIntrospect } from "@/modules/schema/queries/schema.queries";
import { ColumnList } from "@/modules/schema/components/column-list";
import { ColumnEditDialog } from "@/modules/schema/components/column-edit-dialog";
import { DdlViewer } from "@/modules/schema/components/ddl-viewer";
import { DdlEditor } from "@/modules/schema/components/ddl-editor/ddl-editor";
import { ForeignKeyList } from "@/modules/schema/components/foreign-key-list";
import { GenerateCrud } from "@/modules/schema/components/generate-crud";
import { IndexManager } from "@/modules/schema/components/index-manager";
import { TriggerManager } from "@/modules/schema/components/trigger-manager";
import { ErDiagram } from "@/modules/er-diagram/components/er-diagram";
import type { SchemaColumnDto } from "@/modules/schema/types/schema.types";

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
  const [dataRefreshCounter, setDataRefreshCounter] = useState(0);
  const [editingColumn, setEditingColumn] = useState<SchemaColumnDto | null>(null);

  const activeSection = useWorkspaceStore((s) => {
    const tab = s.tabs.find((t) => t.id === tabId);
    return tab?.kind === "db-object" ? tab.data.activeSection : "columns";
  });
  const setSection = useWorkspaceStore((s) => s.setDbObjectSection);

  const isTableOrView = objectType === "table" || objectType === "view";

  const tableInfo = useTableInfo(connectionId, schema, objectName);
  const tableDdl = useTableDdl(
    connectionId,
    schema,
    objectName,
    activeSection === "ddl" || toolbarAction === "ddlEditor",
  );
  const introspect = useIntrospect(
    activeSection === "diagram" ? connectionId : null,
  );

  if (!connectionId) return null;

  const openQueryTab = (sql: string, title: string) => {
    const tab = createQueryTabForObject(connectionId, schema, { title, sql });
    useWorkspaceStore.getState().openTab(tab);
  };

  const handleRefresh = useCallback(() => {
    tableInfo.refetch();
    if (activeSection === "data") {
      setDataRefreshCounter((c) => c + 1);
    }
    if (activeSection === "ddl" || toolbarAction === "ddlEditor") {
      tableDdl.refetch();
    }
  }, [tableInfo, tableDdl, activeSection, toolbarAction]);

  const handleOpenSelect = () => {
    const dialect = getDialectForConnection(connectionId);
    openQueryTab(
      dialect.generateSelect({ schema, table: objectName, limit: 100 }),
      `SELECT ${objectName}`,
    );
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
        columns={tableInfo.data?.columns}
        onRefresh={handleRefresh}
        onOpenSelect={handleOpenSelect}
        onOpenDdl={handleOpenDdl}
        onOpenDdlEditor={() => setToolbarAction("ddlEditor")}
        onGenerateSql={() => setToolbarAction("generateCrud")}
        onOpenQuery={(sql, title) => openQueryTab(sql, title)}
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
            <GenerateCrud
              connectionId={connectionId}
              schema={schema}
              table={objectName}
              columns={tableInfo.data.columns}
            />
          </ObjectSectionLayout>
        )}
        {!toolbarAction && (
          <>
            {activeSection === "data" && isTableOrView && (
              <DataSection
                tabId={tabId}
                connectionId={connectionId}
                schema={schema}
                table={objectName}
                refreshCounter={dataRefreshCounter}
              />
            )}
            {activeSection === "columns" && isTableOrView && tableInfo.data && (
              <ObjectSectionLayout>
                <ColumnList
                  columns={tableInfo.data.columns}
                  onEditColumn={(col) => setEditingColumn(col)}
                />
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
                <ForeignKeyList foreignKeys={tableInfo.data.foreignKeys} connectionId={connectionId} />
              </ObjectSectionLayout>
            )}
            {activeSection === "triggers" && isTableOrView && (
              <ObjectSectionLayout>
                <TriggerManager connectionId={connectionId} schema={schema} table={objectName} />
              </ObjectSectionLayout>
            )}
            {activeSection === "diagram" && isTableOrView && (
              <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
                {introspect.isLoading && (
                  <div className="flex h-full items-center justify-center p-4">
                    <span className="text-[13px] text-[var(--app-text-muted)]">{t("common.states.loading")}</span>
                  </div>
                )}
                {introspect.isError && (
                  <div className="flex h-full items-center justify-center p-4">
                    <span className="text-[13px] text-destructive">{t("common.states.error")}</span>
                  </div>
                )}
                {introspect.data && (
                  <ErDiagram connectionId={connectionId} data={introspect.data} />
                )}
              </div>
            )}
            {activeSection === "ddl" && isTableOrView && (
              <ObjectSectionLayout>
                <DdlViewer
                  ddl={tableDdl.data ?? null}
                  isLoading={tableDdl.isLoading}
                  error={
                    tableDdl.isError
                      ? ((tableDdl.error as { userMessage?: string })?.userMessage ??
                        t("common.states.error"))
                      : null
                  }
                  onOpenInQuery={handleOpenDdl}
                />
              </ObjectSectionLayout>
            )}
            {!isTableOrView && (
              <div className="flex h-full items-center justify-center p-8">
                <p className="text-[13px] text-[var(--app-text-muted)]">
                  {t("dbObject.unsupportedObjectType", { type: objectType })}
                </p>
              </div>
            )}
            {tableInfo.isLoading &&
              (activeSection === "columns" ||
                activeSection === "indexes" ||
                activeSection === "relations") && (
                <div className="flex h-full min-h-0 items-center justify-center">
                  <div className="p-4 text-[13px] text-[var(--app-text-muted)]">
                    {t("common.states.loading")}
                  </div>
                </div>
              )}
            {tableInfo.isError &&
              (activeSection === "columns" ||
                activeSection === "indexes" ||
                activeSection === "relations") && (
                <div className="flex h-full min-h-0 items-center justify-center">
                  <div className="p-4 text-[13px] text-destructive">
                    {(tableInfo.error as { userMessage?: string })?.userMessage ??
                      t("common.states.error")}
                  </div>
                </div>
              )}
          </>
        )}
      </DbObjectWorkspace>

      {editingColumn && (
        <ColumnEditDialog
          column={editingColumn}
          schemaName={schema}
          tableName={objectName}
          connectionId={connectionId}
          onClose={() => setEditingColumn(null)}
          onApplied={() => {
            tableInfo.refetch();
            setDataRefreshCounter((c) => c + 1);
          }}
        />
      )}
    </div>
  );
}
