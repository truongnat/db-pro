import { useCallback, useMemo, useState } from "react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useTranslation } from "@/commons/locales/useTranslation";
import { formatRelativeTime } from "@/commons/utils/date-formatter";

import {
  clearLocalHistory,
  getLocalHistory,
  removeLocalHistoryEntry,
  type LocalHistoryEntry,
} from "../services/local-history";

interface LocalHistoryPanelProps {
  onSelectEntry: (sql: string) => void;
}

export function LocalHistoryPanel({ onSelectEntry }: LocalHistoryPanelProps) {
  const { t } = useTranslation();
  const [search, setSearch] = useState("");
  const [entries, setEntries] = useState<LocalHistoryEntry[]>(() =>
    getLocalHistory(),
  );

  const filtered = useMemo(() => {
    if (!search.trim()) return entries;
    const q = search.toLowerCase();
    return entries.filter((e) => e.sql.toLowerCase().includes(q));
  }, [entries, search]);

  const refresh = useCallback(() => {
    setEntries(getLocalHistory());
  }, []);

  const handleClear = useCallback(() => {
    clearLocalHistory();
    refresh();
  }, [refresh]);

  const handleRemove = useCallback(
    (index: number) => {
      removeLocalHistoryEntry(index);
      refresh();
    },
    [refresh],
  );

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center gap-2 border-b border-[var(--app-border-subtle)] px-3 py-2">
        <Input
          className="flex-1"
          placeholder={t("query.searchHistory")}
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
        <Button
          type="button"
          variant="outline"
          className="rounded-sm border px-2 py-1 text-xs text-[var(--app-text-muted)]"
          onClick={handleClear}
          disabled={entries.length === 0}
        >
          {t("common.actions.clear")}
        </Button>
      </div>

      <div className="flex-1 overflow-auto">
        {filtered.length === 0 && (
          <div className="flex items-center justify-center py-8">
            <p className="text-sm text-[var(--app-text-muted)]">
              {t("common.states.empty")}
            </p>
          </div>
        )}
        {filtered.map((entry, idx) => (
          <div
            key={`${entry.timestamp}-${idx}`}
            className="group flex cursor-pointer items-start gap-2 border-b border-[var(--app-border-subtle)] px-3 py-2 transition-colors hover:bg-background"
            onClick={() => onSelectEntry(entry.sql)}
          >
            <div className="min-w-0 flex-1">
              <pre
                className="overflow-hidden text-ellipsis whitespace-pre-wrap text-xs text-foreground"
                style={{ maxHeight: "3em" }}
              >
                {entry.sql}
              </pre>
              <span className="mt-0.5 text-xs text-muted-foreground">
                {formatRelativeTime(new Date(entry.timestamp))}
              </span>
            </div>
            <Button
              type="button"
              variant="ghost"
              size="sm"
              className="shrink-0 rounded px-1 text-xs text-muted-foreground opacity-0 transition-opacity group-hover:opacity-100"
              onClick={(e) => {
                e.stopPropagation();
                handleRemove(idx);
              }}
              title={t("common.actions.delete")}
            >
              ×
            </Button>
          </div>
        ))}
      </div>
    </div>
  );
}
