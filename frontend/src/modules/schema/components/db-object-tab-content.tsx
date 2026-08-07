import { useMemo, useState } from "react";

import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { useTranslation } from "@/commons/locales/useTranslation";
import { cn } from "@/lib/utils";
import { SchemaDetailPanel } from "./schema-detail-panel";
import type { DetailTab } from "../types/schema.types";
import type { DbObjectSection } from "@/commons/types/workspace.types";

import { useIntrospect } from "@/modules/schema/queries/schema.queries";
import { DataGrid } from "@/modules/data-grid/components/data-grid";
import { Pagination } from "@/modules/data-grid/components/pagination";
import { VisualFilterBuilder } from "@/modules/data-grid/components/visual-filter-builder";
import { useTabGridStateStore } from "@/modules/data-grid/state/tab-grid-state.store";
import {
  useTableRows,
  useUpdateRow,
  useDeleteRow,
} from "@/modules/data-grid/queries/data-grid.queries";
import type { CellValue, FetchRowsRequest, GridSort } from "@/modules/data-grid/types/data-grid.types";

interface DbObjectTabContentProps {
  tabId: string;
  connectionId: string | null;
  schema: string;
  objectName: string;
  objectType: "table" | "view" | "function" | "sequence" | "type";
}

const SECTIONS: { id: DbObjectSection; labelKey: string }[] = [
  { id: "structure", labelKey: "dbObject.sections.structure" },
  { id: "data", labelKey: "dbObject.sections.data" },
  { id: "indexes", labelKey: "dbObject.sections.indexes" },
  { id: "relations", labelKey: "dbObject.sections.relations" },
  { id: "ddl", labelKey: "dbObject.sections.ddl" },
  { id: "triggers", labelKey: "dbObject.sections.triggers" },
];

function DataSection({ tabId, connectionId, schema, table }: { tabId: string; connectionId: string; schema: string; table: string }) {
  const tabState = useTabGridStateStore((s) => s.states[tabId]) ?? {
    filters: [], sorts: [] as GridSort[], page: 1, pageSize: 50,
    editingCell: null, frozenColumns: [] as string[], chartConfig: null,
  };
  const store = useTabGridStateStore.getState();

  const filters = tabState.filters;
  const sorts = tabState.sorts;
  const page = tabState.page;
  const pageSize = tabState.pageSize;
  const editingCell = tabState.editingCell;
  const frozenColumns = tabState.frozenColumns;

  const introspect = useIntrospect(connectionId);

  const pkColumns = useMemo(() => {
    if (!introspect.data) return [];
    return (
      introspect.data.primaryKeys
        .filter((pk) => pk.schema === schema && pk.tableName === table)
        .flatMap((pk) => pk.columns)
    );
  }, [introspect.data, schema, table]);

  const request: FetchRowsRequest = { schema, table, filters, sorts, page, pageSize };

  const query = useTableRows(connectionId, request);
  const updateRow = useUpdateRow(connectionId, request);
  const deleteRow = useDeleteRow(connectionId, request);

  const columns = query.data?.columns ?? [];
  const rows = query.data?.rows ?? [];
  const totalCount = query.data?.totalCount ?? 0;

  const handleSort = (column: string) => {
    const existing = sorts.find((s) => s.column === column);
    let newSorts: GridSort[];
    if (!existing) {
      newSorts = [{ column, direction: "asc" }];
    } else if (existing.direction === "asc") {
      newSorts = [{ column, direction: "desc" }];
    } else {
      newSorts = [];
    }
    store.setSorts(tabId, newSorts);
  };

  const handleCellSave = (rowIdx: number, colIdx: number, value: CellValue) => {
    if (!pkColumns.length) return;
    const row = rows[rowIdx];
    const col = columns[colIdx];
    const colNameToIdx = new Map(columns.map((c, i) => [c.name, i]));
    const updatedRow = [...row];
    updatedRow[colIdx] = value;
    const pkValues = pkColumns.map((pkCol) => {
      const idx = colNameToIdx.get(pkCol);
      return idx !== undefined ? row[idx] : { type: "null" as const };
    });
    updateRow.mutate({
      schema,
      table,
      columns: columns.map((c) => c.name),
      values: updatedRow,
      pkColumns,
      pkValues,
    });
  };

  const handleDeleteRow = (rowIdx: number) => {
    if (!pkColumns.length) return;
    const row = rows[rowIdx];
    const colNameToIdx = new Map(columns.map((c, i) => [c.name, i]));
    const pkValues = pkColumns.map((pkCol) => {
      const idx = colNameToIdx.get(pkCol);
      return idx !== undefined ? row[idx] : { type: "null" as const };
    });
    deleteRow.mutate({
      schema,
      table,
      columns: columns.map((c) => c.name),
      values: row,
      pkColumns,
      pkValues,
    });
  };

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      {columns.length > 0 && (
        <VisualFilterBuilder
          columns={columns}
          filters={filters}
          onAddFilter={(f) => store.addFilter(tabId, f)}
          onRemoveFilter={(i) => store.removeFilter(tabId, i)}
        />
      )}

      <div className="relative flex-1 overflow-hidden">
        <DataGrid
          columns={columns}
          rows={rows}
          sorts={sorts}
          onSort={handleSort}
          editingCell={editingCell}
          onEditCell={(c) => store.setEditingCell(tabId, c)}
          onCellSave={handleCellSave}
          onDeleteRow={handleDeleteRow}
          isDeleting={deleteRow.isPending}
          isLoading={query.isFetching && !query.isPlaceholderData}
          pkColumns={pkColumns}
          frozenColumns={frozenColumns}
          onToggleFreezeColumn={(c) => store.toggleFrozenColumn(tabId, c)}
        />
      </div>

      <Pagination
        page={page}
        pageSize={pageSize}
        totalCount={totalCount}
        onPageChange={(p) => store.setPage(tabId, p)}
        onPageSizeChange={(s) => store.setPageSize(tabId, s)}
      />
    </div>
  );
}

export function DbObjectTabContent({
  tabId,
  connectionId,
  schema,
  objectName,
  objectType,
}: DbObjectTabContentProps) {
  const { t } = useTranslation();
  const [detailTab, setDetailTab] = useState<DetailTab>("columns");

  const activeSection = useWorkspaceStore((s) => {
    const tab = s.tabs.find((t) => t.id === tabId);
    return tab?.kind === "db-object" ? tab.data.activeSection : "structure";
  });
  const setSection = useWorkspaceStore((s) => s.setDbObjectSection);

  if (!connectionId) return null;

  const isTableOrView = objectType === "table" || objectType === "view";

  return (
    <div className="flex h-full flex-col">
      <div className="flex border-b border-border">
        {SECTIONS.map((section) => (
          <button
            key={section.id}
            type="button"
            className={cn(
              "px-3 py-2 text-xs font-medium transition-colors hover:bg-[var(--app-hover)]",
              activeSection === section.id
                ? "border-b-2 border-primary text-foreground"
                : "text-muted-foreground",
            )}
            onClick={() => setSection(tabId, section.id)}
          >
            {t(section.labelKey)}
          </button>
        ))}
      </div>

      <div className="flex-1 overflow-hidden">
        {activeSection === "structure" && isTableOrView && (
          <SchemaDetailPanel
            connectionId={connectionId}
            schema={schema}
            table={objectName}
            nodeType={objectType === "view" ? "view" : "table"}
            activeTab={detailTab}
            onTabChange={setDetailTab}
          />
        )}
        {activeSection === "data" && isTableOrView && (
          <DataSection tabId={tabId} connectionId={connectionId} schema={schema} table={objectName} />
        )}
        {activeSection === "indexes" && isTableOrView && (
          <SchemaDetailPanel
            connectionId={connectionId}
            schema={schema}
            table={objectName}
            nodeType={objectType === "view" ? "view" : "table"}
            activeTab="indexes"
            onTabChange={setDetailTab}
          />
        )}
        {activeSection === "relations" && isTableOrView && (
          <SchemaDetailPanel
            connectionId={connectionId}
            schema={schema}
            table={objectName}
            nodeType={objectType === "view" ? "view" : "table"}
            activeTab="foreignKeys"
            onTabChange={setDetailTab}
          />
        )}
        {activeSection === "ddl" && isTableOrView && (
          <SchemaDetailPanel
            connectionId={connectionId}
            schema={schema}
            table={objectName}
            nodeType={objectType === "view" ? "view" : "table"}
            activeTab="ddl"
            onTabChange={setDetailTab}
          />
        )}
        {activeSection === "triggers" && isTableOrView && (
          <SchemaDetailPanel
            connectionId={connectionId}
            schema={schema}
            table={objectName}
            nodeType={objectType === "view" ? "view" : "table"}
            activeTab="triggers"
            onTabChange={setDetailTab}
          />
        )}
        {!isTableOrView && (
          <div className="flex h-full items-center justify-center p-8">
            <p className="text-sm text-muted-foreground">
              {t("dbObject.unsupportedObjectType", { type: objectType })}
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
