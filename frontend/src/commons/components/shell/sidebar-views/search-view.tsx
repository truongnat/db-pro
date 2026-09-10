import { useEffect, useMemo, useRef, useState } from "react";

import { useVirtualizer } from "@tanstack/react-virtual";
import { useTranslation } from "@/commons/locales/useTranslation";
import { useConnectionStore } from "@/commons/stores/connection.store";
import { useSidebarTabOps } from "@/commons/hooks/use-sidebar-tab-ops";
import { Input } from "@/components/ui/input";
import { useIntrospect } from "@/modules/schema/queries/schema.queries";
import { Table2, Columns3 } from "lucide-react";

type SearchResult = {
  kind: "table" | "view";
  name: string;
  schema: string;
};

export function SearchView() {
  const { t } = useTranslation();
  const [query, setQuery] = useState("");
  const explorerConnectionId = useConnectionStore((s) => s.explorerConnectionId);
  const introspect = useIntrospect(explorerConnectionId);
  const { openSchemaPreview, openTableData } = useSidebarTabOps();

  const [debouncedQuery, setDebouncedQuery] = useState("");
  useEffect(() => {
    const timer = window.setTimeout(() => setDebouncedQuery(query), 120);
    return () => window.clearTimeout(timer);
  }, [query]);

  const searchCatalog = useMemo<SearchResult[]>(
    () => [
      ...(introspect.data?.tables ?? []).map(({ name, schema }) => ({
        kind: "table" as const,
        name,
        schema,
      })),
      ...(introspect.data?.views ?? []).map(({ name, schema }) => ({
        kind: "view" as const,
        name,
        schema,
      })),
    ],
    [introspect.data?.tables, introspect.data?.views],
  );

  const filteredResults = useMemo(() => {
    const lowerQuery = debouncedQuery.trim().toLowerCase();
    if (!lowerQuery) return [];
    return searchCatalog.filter(
      ({ name, schema }) =>
        name.toLowerCase().includes(lowerQuery) || schema.toLowerCase().includes(lowerQuery),
    );
  }, [debouncedQuery, searchCatalog]);

  const resultsRef = useRef<HTMLDivElement>(null);
  const virtualizer = useVirtualizer({
    count: filteredResults.length,
    getScrollElement: () => resultsRef.current,
    estimateSize: () => 28,
    overscan: 8,
  });

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

      {explorerConnectionId && debouncedQuery && (
        <>
          <p className="px-2 text-[11px] text-[var(--text-tertiary)]">
            {t("shell.sidebar.searchResultCount", { count: filteredResults.length })}
          </p>
          {filteredResults.length > 0 ? (
            <div ref={resultsRef} className="max-h-64 overflow-y-auto">
              <div className="relative" style={{ height: virtualizer.getTotalSize() }}>
                {virtualizer.getVirtualItems().map((virtualRow) => {
                  const result = filteredResults[virtualRow.index];
                  const Icon = result.kind === "table" ? Table2 : Columns3;
                  return (
                    <button
                      key={`${result.kind}:${result.schema}.${result.name}`}
                      type="button"
                      title={`${result.schema}.${result.name}`}
                      className="absolute left-0 flex h-7 w-full items-center gap-2 rounded-md px-2 text-xs text-[var(--text-secondary)] transition-colors hover:bg-[var(--surface-hover)] hover:text-foreground"
                      style={{ transform: `translateY(${virtualRow.start}px)` }}
                      onClick={() =>
                        openSchemaPreview(
                          explorerConnectionId,
                          result.schema,
                          result.name,
                          result.kind,
                        )
                      }
                      onDoubleClick={() =>
                        openTableData(explorerConnectionId, result.schema, result.name, result.kind)
                      }
                    >
                      <Icon className="h-3 w-3 shrink-0 text-primary" />
                      <span className="truncate">{result.name}</span>
                      <span className="ml-auto text-[11px] text-[var(--text-tertiary)]">
                        {result.schema}
                      </span>
                    </button>
                  );
                })}
              </div>
            </div>
          ) : (
            <p className="px-2 py-1 text-xs text-[var(--text-tertiary)]">
              {t("common.states.empty")}
            </p>
          )}
        </>
      )}
    </div>
  );
}
