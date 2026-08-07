import { useCallback, useMemo, useState } from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useTranslation } from "@/commons/locales/useTranslation";
import { useQueryHistoryStore } from "@/commons/stores/query-history.store";

import type { QueryHistoryEntry } from "../types/query.types";

interface QueryHistoryPanelProps {
  entries: QueryHistoryEntry[];
  search: string;
  onSearchChange: (search: string) => void;
  onSelectEntry: (sql: string) => void;
  onOpenInNewTab?: (sql: string) => void;
  isLoading: boolean;
}

export function QueryHistoryPanel({
  entries,
  search,
  onSearchChange,
  onSelectEntry,
  onOpenInNewTab,
  isLoading,
}: QueryHistoryPanelProps) {
  const { t } = useTranslation();
  const toggleFavorite = useQueryHistoryStore((s) => s.toggleFavorite);
  const isFavorite = useQueryHistoryStore((s) => s.isFavorite);
  const [showFavoritesOnly, setShowFavoritesOnly] = useState(false);

  const filtered = useMemo(() => {
    let result = entries;
    if (search) {
      const q = search.toLowerCase();
      result = result.filter(
        (e) =>
          e.sql.toLowerCase().includes(q) ||
          (e.database && e.database.toLowerCase().includes(q)) ||
          (e.schema && e.schema.toLowerCase().includes(q)),
      );
    }
    if (showFavoritesOnly) {
      result = result.filter((e) => isFavorite(e.id));
    }
    return result;
  }, [entries, search, showFavoritesOnly, isFavorite]);

  const handleCopy = useCallback((sql: string) => {
    navigator.clipboard.writeText(sql);
  }, []);

  const handleRerun = useCallback(
    (sql: string) => {
      onSelectEntry(sql);
    },
    [onSelectEntry],
  );

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center gap-1 border-b border-[var(--app-border-subtle)] p-2">
        <Input
          value={search}
          onChange={(e) => onSearchChange(e.target.value)}
          placeholder={t("query.searchHistory")}
          className="flex-1 text-xs"
        />
        <Button
          type="button"
          variant={showFavoritesOnly ? "default" : "ghost"}
          size="sm"
          className="shrink-0 text-xs"
          onClick={() => setShowFavoritesOnly(!showFavoritesOnly)}
        >
          {showFavoritesOnly ? "★" : "☆"}
        </Button>
      </div>

      <div className="flex-1 overflow-auto">
        {isLoading ? (
          <div className="flex items-center justify-center py-8">
            <p className="text-[var(--app-text-muted)]">{t("common.states.loading")}</p>
          </div>
        ) : filtered.length === 0 ? (
          <div className="flex items-center justify-center py-8">
            <p className="text-[var(--app-text-muted)]">{t("common.states.empty")}</p>
          </div>
        ) : (
          filtered.map((entry) => (
            <HistoryEntryRow
              key={entry.id}
              entry={entry}
              isFavorite={isFavorite(entry.id)}
              onToggleFavorite={() => toggleFavorite(entry.id)}
              onSelect={() => onSelectEntry(entry.sql)}
              onRerun={() => handleRerun(entry.sql)}
              onCopy={() => handleCopy(entry.sql)}
              onOpenInNewTab={onOpenInNewTab ? () => onOpenInNewTab(entry.sql) : undefined}
            />
          ))
        )}
      </div>
    </div>
  );
}

/* ─── History Entry Row ─────────────────────────────────────────── */

function HistoryEntryRow({
  entry,
  isFavorite: fav,
  onToggleFavorite,
  onSelect,
  onRerun,
  onCopy,
  onOpenInNewTab,
}: {
  entry: QueryHistoryEntry;
  isFavorite: boolean;
  onToggleFavorite: () => void;
  onSelect: () => void;
  onRerun: () => void;
  onCopy: () => void;
  onOpenInNewTab?: () => void;
}) {
  const { t } = useTranslation();
  const isError = entry.status === "error";

  return (
    <div className="group flex flex-col border-b border-[var(--app-border-subtle)] px-3 py-2 transition-colors hover:bg-background">
      {/* SQL line */}
      <div className="flex items-start gap-2">
        <button
          type="button"
          className="shrink-0 text-xs text-[var(--app-text-muted)] hover:text-warning"
          onClick={onToggleFavorite}
          title={fav ? t("query.unfavorite") : t("query.favorite")}
        >
          {fav ? "★" : "☆"}
        </button>
        <div
          className="min-w-0 flex-1 cursor-pointer"
          onClick={onSelect}
        >
          <div
            className={`truncate font-mono text-sm ${isError ? "text-destructive" : "text-foreground"}`}
            title={entry.sql}
          >
            {entry.sql}
          </div>
        </div>
      </div>

      {/* Meta row */}
      <div className="ml-6 mt-1 flex flex-wrap items-center gap-2 text-xs text-[var(--app-text-muted)]">
        <Badge
          variant={isError ? "destructive" : "secondary"}
          className="px-1.5 py-0 text-[10px]"
        >
          {isError ? t("query.statusError") : t("query.statusSuccess")}
        </Badge>
        <span>{t("query.duration", { duration: entry.durationMs })}</span>
        <span>{t("query.rowsAffected", { count: entry.rowCount })}</span>
        <span>{new Date(entry.executedAt).toLocaleTimeString()}</span>
        {entry.database && (
          <span className="text-[var(--app-text-dim)]">
            {entry.database}
            {entry.schema ? `.${entry.schema}` : ""}
          </span>
        )}
      </div>

      {/* Action buttons (visible on hover) */}
      <div className="ml-6 mt-1 flex gap-1 opacity-0 transition-opacity group-hover:opacity-100">
        <Button
          type="button"
          variant="ghost"
          size="sm"
          className="h-6 px-2 text-[10px]"
          onClick={onRerun}
        >
          {t("query.rerun")}
        </Button>
        <Button
          type="button"
          variant="ghost"
          size="sm"
          className="h-6 px-2 text-[10px]"
          onClick={onCopy}
        >
          {t("common.actions.copy")}
        </Button>
        {onOpenInNewTab && (
          <Button
            type="button"
            variant="ghost"
            size="sm"
            className="h-6 px-2 text-[10px]"
            onClick={onOpenInNewTab}
          >
            {t("query.openInNewTab")}
          </Button>
        )}
      </div>
    </div>
  );
}
