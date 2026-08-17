import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Search, Table2 } from "lucide-react";

import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";

import type { ErGraphModel } from "../renderer/types";
import { findTableMatches } from "../utils/overview-search";
import { suggestStartingPoints } from "../utils/neighborhood";

interface ErSearchEntryProps {
  model: ErGraphModel;
  onSelectTable: (tableKey: string) => void;
}

/**
 * Search-first entry surface for large schemas (Gate 4 Slice B).
 *
 * Renders when `shouldEnterLargeSchemaFlow` is true and the phase is "search".
 * No graph renderer (React Flow or Cytoscape) is mounted — the user must
 * explicitly select a table to transition to the neighborhood phase.
 *
 * Search typing filters metadata only; it never triggers graph node
 * construction, layout computation, or renderer mounting.
 */
export function ErSearchEntry({ model, onSelectTable }: ErSearchEntryProps) {
  const [query, setQuery] = useState("");
  const [highlightedIndex, setHighlightedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLUListElement>(null);

  // Deterministic filtered results — model order, case-insensitive match.
  const results = useMemo(() => {
    if (!query.trim()) return [];
    return findTableMatches(model, query);
  }, [model, query]);

  // Suggested starting points when query is empty — hub tables by FK degree.
  const suggestions = useMemo(() => {
    if (query.trim()) return [];
    return suggestStartingPoints(
      model.adjacency,
      model.tables.map((t) => ({ name: t.label, schema: t.schema })),
    );
  }, [model, query]);

  // Reset highlight when results change.
  useEffect(() => {
    setHighlightedIndex(0);
  }, [query]);

  // Auto-focus the search input on mount.
  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  // Scroll the highlighted item into view.
  useEffect(() => {
    const list = listRef.current;
    if (!list) return;
    const item = list.children[highlightedIndex] as HTMLElement | undefined;
    item?.scrollIntoView({ block: "nearest" });
  }, [highlightedIndex]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      const items = results.length > 0 ? results : suggestions;

      switch (e.key) {
        case "ArrowDown":
          e.preventDefault();
          setHighlightedIndex((i) => Math.min(i + 1, items.length - 1));
          break;
        case "ArrowUp":
          e.preventDefault();
          setHighlightedIndex((i) => Math.max(i - 1, 0));
          break;
        case "Enter":
          e.preventDefault();
          if (results.length > 0 && highlightedIndex < results.length) {
            onSelectTable(results[highlightedIndex]);
          }
          break;
        case "Escape":
          e.preventDefault();
          if (query) {
            setQuery("");
            setHighlightedIndex(0);
          } else {
            inputRef.current?.blur();
          }
          break;
      }
    },
    [results, suggestions, highlightedIndex, onSelectTable, query],
  );

  const displayItems = results.length > 0 ? results : suggestions;
  const isShowingResults = results.length > 0;

  return (
    <div className="flex min-h-0 flex-1 flex-col items-center justify-center gap-4 p-8">
      <div className="flex w-full max-w-md flex-col gap-3">
        {/* Search input — visually prominent */}
        <div className="relative">
          <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-[var(--text-secondary)]" />
          <Input
            ref={inputRef}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Search tables..."
            className="h-10 rounded-md pl-9 text-sm"
            data-testid="er-search-input"
          />
        </div>

        {/* Schema stats */}
        <div className="flex items-center gap-2 text-xs text-[var(--text-secondary)]">
          <Badge variant="outline" className="h-6 gap-1 text-[11px]">
            <Table2 className="h-3 w-3" />
            {model.tables.length} tables
          </Badge>
          <span>{model.relations.length} relations</span>
        </div>

        {/* Results / suggestions list */}
        {displayItems.length > 0 && (
          <ul
            ref={listRef}
            className="flex max-h-64 flex-col gap-0.5 overflow-y-auto rounded-md border bg-popover p-1"
            role="listbox"
            data-testid="er-search-results"
          >
            {displayItems.map((tableKey, i) => {
              const table = model.tables.find((t) => t.id === tableKey);
              const isHighlighted = i === highlightedIndex;

              return (
                <li
                  key={tableKey}
                  role="option"
                  aria-selected={isHighlighted}
                  className={`cursor-pointer rounded px-2 py-1.5 text-sm ${
                    isHighlighted ? "bg-accent text-accent-foreground" : "hover:bg-accent/50"
                  }`}
                  data-testid="er-search-result"
                  onClick={() => {
                    if (isShowingResults) {
                      onSelectTable(tableKey);
                    }
                  }}
                  onMouseEnter={() => setHighlightedIndex(i)}
                >
                  <span className="font-medium">{table?.label ?? tableKey}</span>
                  {table && (
                    <span className="ml-2 text-xs text-[var(--text-secondary)]">
                      {table.columnCount} cols · {table.fkCount} FK
                    </span>
                  )}
                  {!isShowingResults && (
                    <span className="ml-2 text-xs text-[var(--text-secondary)]">suggested</span>
                  )}
                </li>
              );
            })}
          </ul>
        )}

        {/* Empty state */}
        {query.trim() && results.length === 0 && (
          <p className="text-center text-xs text-[var(--text-secondary)]">
            No tables match "{query}"
          </p>
        )}
      </div>
    </div>
  );
}
