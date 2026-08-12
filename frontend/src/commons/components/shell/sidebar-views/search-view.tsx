import { useState } from "react";

import { useTranslation } from "@/commons/locales/useTranslation";
import { useConnectionStore } from "@/commons/stores/connection.store";
import { useSidebarTabOps } from "@/commons/hooks/use-sidebar-tab-ops";
import { Input } from "@/components/ui/input";
import { useIntrospect } from "@/modules/schema/queries/schema.queries";
import { Table2, Columns3 } from "lucide-react";

export function SearchView() {
  const { t } = useTranslation();
  const [query, setQuery] = useState("");
  const explorerConnectionId = useConnectionStore((s) => s.explorerConnectionId);
  const introspect = useIntrospect(explorerConnectionId);
  const { openSchemaPreview, openTableData } = useSidebarTabOps();

  const lowerQuery = query.toLowerCase();
  const filteredTables =
    introspect.data?.tables.filter((t) => t.name.toLowerCase().includes(lowerQuery)) ?? [];
  const filteredViews =
    introspect.data?.views.filter((v) => v.name.toLowerCase().includes(lowerQuery)) ?? [];

  return (
    <div className="flex min-h-0 flex-col gap-2">
      <Input
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        placeholder={t("shell.sidebar.searchObjects")}
        className="h-7 text-xs"
      />

      {!explorerConnectionId && (
        <p className="px-2 py-1 text-xs text-[var(--text-tertiary)]">
          {t("shell.sidebar.connectFirst")}
        </p>
      )}

      {explorerConnectionId && query && (
        <div className="flex flex-col gap-0.5 overflow-y-auto">
          {filteredTables.map((table) => (
            <button
              key={`${table.schema}.${table.name}`}
              type="button"
              title={`${table.schema}.${table.name}`}
              className="flex w-full items-center gap-2 rounded-md px-2 py-1 text-xs text-[var(--text-secondary)] transition-colors hover:bg-[var(--surface-hover)] hover:text-foreground"
              onClick={() =>
                openSchemaPreview(explorerConnectionId, table.schema, table.name, "table")
              }
              onDoubleClick={() =>
                openTableData(explorerConnectionId, table.schema, table.name, "table")
              }
            >
              <Table2 className="h-3 w-3 shrink-0 text-primary" />
              <span className="truncate">{table.name}</span>
              <span className="ml-auto text-[11px] text-[var(--text-tertiary)]">
                {table.schema}
              </span>
            </button>
          ))}
          {filteredViews.map((view) => (
            <button
              key={`${view.schema}.${view.name}`}
              type="button"
              title={`${view.schema}.${view.name}`}
              className="flex w-full items-center gap-2 rounded-md px-2 py-1 text-xs text-[var(--text-secondary)] transition-colors hover:bg-[var(--surface-hover)] hover:text-foreground"
              onClick={() =>
                openSchemaPreview(explorerConnectionId, view.schema, view.name, "view")
              }
              onDoubleClick={() =>
                openTableData(explorerConnectionId, view.schema, view.name, "view")
              }
            >
              <Columns3 className="h-3 w-3 shrink-0 text-primary" />
              <span className="truncate">{view.name}</span>
              <span className="ml-auto text-[11px] text-[var(--text-tertiary)]">{view.schema}</span>
            </button>
          ))}
          {filteredTables.length === 0 && filteredViews.length === 0 && (
            <p className="px-2 py-1 text-xs text-[var(--text-tertiary)]">
              {t("common.states.empty")}
            </p>
          )}
        </div>
      )}
    </div>
  );
}
