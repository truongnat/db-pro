import { useTranslation } from "@/commons/locales/useTranslation";

import { useTableDdl, useTableInfo } from "../queries/schema.queries";
import type { DetailTab } from "../types/schema.types";

import { ColumnList } from "./column-list";
import { DdlEditor } from "./ddl-editor/ddl-editor";
import { DdlViewer } from "./ddl-viewer";
import { ForeignKeyList } from "./foreign-key-list";
import { GenerateCrud } from "./generate-crud";
import { IndexManager } from "./index-manager";
import { IndexList } from "./index-list";
import { TriggerManager } from "./trigger-manager";

interface SchemaDetailPanelProps {
  connectionId: string;
  schema: string | null;
  table: string | null;
  nodeType: "table" | "view" | null;
  activeTab: DetailTab;
  onTabChange: (tab: DetailTab) => void;
}

const TABLE_TABS: { key: DetailTab; labelKey: string }[] = [
  { key: "columns", labelKey: "schema.columns" },
  { key: "indexes", labelKey: "schema.indexes" },
  { key: "foreignKeys", labelKey: "schema.foreignKeys" },
  { key: "ddl", labelKey: "schema.ddl" },
  { key: "ddlEditor", labelKey: "schema.ddlEditor" },
  { key: "generateCrud", labelKey: "dataGrid.generateCrud" },
  { key: "triggers", labelKey: "schema.triggers" },
];

const VIEW_TABS: { key: DetailTab; labelKey: string }[] = [
  { key: "columns", labelKey: "schema.columns" },
  { key: "ddl", labelKey: "schema.ddl" },
];

export function SchemaDetailPanel({
  connectionId,
  schema,
  table,
  nodeType,
  activeTab,
  onTabChange,
}: SchemaDetailPanelProps) {
  const { t } = useTranslation();

  if (!schema || !table || !nodeType) {
    return (
      <div
        className="flex flex-1 items-center justify-center p-8 text-sm"
        style={{ color: "var(--color-text-secondary)" }}
      >
        {t("schema.selectTable")}
      </div>
    );
  }

  const tabs = nodeType === "view" ? VIEW_TABS : TABLE_TABS;
  const tableInfo = useTableInfo(connectionId, schema, table);
  const tableDdl = useTableDdl(connectionId, schema, table, activeTab === "ddl");

  const tabStyle = (isActive: boolean): React.CSSProperties => ({
    borderColor: isActive ? "var(--color-primary, #3b82f6)" : "transparent",
    color: isActive ? "var(--color-primary, #3b82f6)" : "var(--color-text-secondary)",
    borderBottomWidth: "2px",
  });

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      <div
        className="flex gap-1 border-b px-3"
        style={{ borderColor: "var(--color-border)" }}
      >
        {tabs.map((tab) => (
          <button
            key={tab.key}
            className="px-3 py-2 text-sm transition-colors hover:opacity-80"
            style={tabStyle(activeTab === tab.key)}
            onClick={() => onTabChange(tab.key)}
            type="button"
          >
            {t(tab.labelKey)}
          </button>
        ))}
      </div>

      <div className="flex-1 overflow-auto">
        {tableInfo.isError && (
          <div
            className="p-4 text-sm"
            style={{ color: "var(--color-error, #ef4444)" }}
          >
            {(tableInfo.error as { userMessage?: string })?.userMessage ?? t("common.states.error")}
          </div>
        )}

        {activeTab === "columns" && tableInfo.data && (
          <ColumnList columns={tableInfo.data.columns} />
        )}

        {activeTab === "indexes" && tableInfo.data && (
          <IndexManager
            connectionId={connectionId}
            schema={schema}
            table={table}
            columns={tableInfo.data.columns}
            indexes={tableInfo.data.indexes}
          />
        )}

        {activeTab === "foreignKeys" && tableInfo.data && (
          <ForeignKeyList foreignKeys={tableInfo.data.foreignKeys} />
        )}

        {activeTab === "ddl" && (
          <DdlViewer
            ddl={tableDdl.data ?? null}
            isLoading={tableDdl.isLoading}
            error={
              tableDdl.isError
                ? (tableDdl.error as { userMessage?: string })?.userMessage ?? t("common.states.error")
                : null
            }
          />
        )}

        {activeTab === "ddlEditor" && (
          <DdlEditor connectionId={connectionId} schema={schema} table={table} />
        )}

        {activeTab === "generateCrud" && tableInfo.data && (
          <GenerateCrud
            schema={schema}
            table={table}
            columns={tableInfo.data.columns}
          />
        )}

        {activeTab === "triggers" && (
          <TriggerManager
            connectionId={connectionId}
            schema={schema}
            table={table}
          />
        )}

        {tableInfo.isLoading && (activeTab === "columns" || activeTab === "indexes" || activeTab === "foreignKeys") && (
          <div
            className="p-4 text-sm"
            style={{ color: "var(--color-text-secondary)" }}
          >
            {t("common.states.loading")}
          </div>
        )}
      </div>
    </div>
  );
}
