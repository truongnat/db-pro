import { useMemo } from "react";

import { useConnectionStore } from "@/commons/stores/connection.store";
import { useTranslation } from "@/commons/locales/useTranslation";

import {
  SchemaDetailPanel,
} from "../components/schema-detail-panel";
import { SchemaToolbar } from "../components/schema-toolbar";
import { SchemaTree } from "../components/schema-tree";
import { useInvalidateSchemaCache, useIntrospect } from "../queries/schema.queries";
import { useSchemaModuleStore } from "../state/schema.store";
import { buildTreeData } from "../types/schema.types";

export function SchemaPage() {
  const { t } = useTranslation();
  const activeConnectionId = useConnectionStore((s) => s.activeConnectionId);

  const searchQuery = useSchemaModuleStore((s) => s.searchQuery);
  const setSearchQuery = useSchemaModuleStore((s) => s.setSearchQuery);
  const expandedNodes = useSchemaModuleStore((s) => s.expandedNodes);
  const toggleNode = useSchemaModuleStore((s) => s.toggleNode);
  const selectedSchema = useSchemaModuleStore((s) => s.selectedSchema);
  const selectedTable = useSchemaModuleStore((s) => s.selectedTable);
  const selectedNodeType = useSchemaModuleStore((s) => s.selectedNodeType);
  const setSelectedTable = useSchemaModuleStore((s) => s.setSelectedTable);
  const activeTab = useSchemaModuleStore((s) => s.activeTab);
  const setActiveTab = useSchemaModuleStore((s) => s.setActiveTab);

  const introspect = useIntrospect(activeConnectionId);
  const invalidateCache = useInvalidateSchemaCache(activeConnectionId);

  const treeNodes = useMemo(() => {
    if (!introspect.data) return [];
    return buildTreeData(introspect.data, searchQuery);
  }, [introspect.data, searchQuery]);

  const tableCount = useMemo(() => {
    if (!introspect.data) return 0;
    const query = searchQuery.toLowerCase().trim();
    const tables = introspect.data.tables;
    const views = introspect.data.views;
    if (!query) return tables.length + views.length;
    return (
      tables.filter((t) => t.name.toLowerCase().includes(query)).length +
      views.filter((v) => v.name.toLowerCase().includes(query)).length
    );
  }, [introspect.data, searchQuery]);

  const selectedNodeId =
    selectedSchema && selectedTable
      ? `${selectedNodeType}:${selectedSchema}:${selectedTable}`
      : null;

  if (!activeConnectionId) {
    return (
      <div
        className="flex flex-1 items-center justify-center text-sm"
        style={{ color: "var(--color-text-secondary)" }}
      >
        {t("schema.connectFirst")}
      </div>
    );
  }

  if (introspect.isLoading) {
    return (
      <div
        className="flex flex-1 items-center justify-center text-sm"
        style={{ color: "var(--color-text-secondary)" }}
      >
        {t("common.states.loading")}
      </div>
    );
  }

  if (introspect.isError) {
    return (
      <div
        className="flex flex-1 flex-col items-center justify-center gap-3 text-sm"
        style={{ color: "var(--color-error, #ef4444)" }}
      >
        <span>
          {(introspect.error as { userMessage?: string })?.userMessage ?? t("error.introspection.failed")}
        </span>
        <button
          className="rounded-[var(--radius-sm)] border px-3 py-1 text-sm transition-colors hover:bg-[var(--color-bg)]"
          style={{
            borderColor: "var(--color-border)",
            color: "var(--color-text)",
          }}
          onClick={() => introspect.refetch()}
          type="button"
        >
          {t("common.actions.retry")}
        </button>
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col">
      <SchemaToolbar
        searchQuery={searchQuery}
        onSearchChange={setSearchQuery}
        onRefresh={() => invalidateCache.mutate()}
        isRefreshing={invalidateCache.isPending}
        tableCount={tableCount}
      />

      <div className="flex flex-1 overflow-hidden">
        <div
          className="flex flex-col overflow-hidden border-r"
          style={{ width: "30%", minWidth: "200px", borderColor: "var(--color-border)" }}
        >
          <SchemaTree
            treeNodes={treeNodes}
            expandedNodes={expandedNodes}
            selectedNodeId={selectedNodeId}
            onToggleNode={toggleNode}
            onSelectNode={(schema, name, type) => setSelectedTable(schema, name, type)}
          />
        </div>

        <div className="flex flex-1 flex-col overflow-hidden">
          <SchemaDetailPanel
            connectionId={activeConnectionId}
            schema={selectedSchema}
            table={selectedTable}
            nodeType={selectedNodeType}
            activeTab={activeTab}
            onTabChange={setActiveTab}
          />
        </div>
      </div>
    </div>
  );
}
