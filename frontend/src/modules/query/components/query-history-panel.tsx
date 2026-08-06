import { Button } from "@/components/ui/button";
import { useTranslation } from "@/commons/locales/useTranslation";

import type { QueryHistoryEntry } from "../types/query.types";

interface QueryHistoryPanelProps {
  entries: QueryHistoryEntry[];
  search: string;
  onSearchChange: (search: string) => void;
  onSelectEntry: (sql: string) => void;
  isLoading: boolean;
}

export function QueryHistoryPanel({
  entries,
  search,
  onSearchChange,
  onSelectEntry,
  isLoading,
}: QueryHistoryPanelProps) {
  const { t } = useTranslation();

  const filtered = search
    ? entries.filter((e) =>
        e.sql.toLowerCase().includes(search.toLowerCase()),
      )
    : entries;

  return (
    <div className="flex h-full flex-col">
      <div className="border-b border-border p-2">
        <input
          type="text"
          value={search}
          onChange={(e) => onSearchChange(e.target.value)}
          placeholder={t("query.searchHistory")}
          className="w-full rounded-sm border border-border bg-background px-3 py-1.5 text-sm text-foreground outline-none transition-colors focus:border-primary"
        />
      </div>

      <div className="flex-1 overflow-auto">
        {isLoading ? (
          <div className="flex items-center justify-center py-8">
            <p className="text-muted-foreground">
              {t("common.states.loading")}
            </p>
          </div>
        ) : filtered.length === 0 ? (
          <div className="flex items-center justify-center py-8">
            <p className="text-muted-foreground">
              {t("common.states.empty")}
            </p>
          </div>
        ) : (
          filtered.map((entry) => (
            <Button
              key={entry.id}
              type="button"
              variant="ghost"
              className="flex w-full flex-col items-start justify-center rounded-none border-b border-border px-3 py-2 text-left transition-colors"
              onClick={() => onSelectEntry(entry.sql)}
            >
              <div
                className="truncate text-sm font-mono text-foreground"
                title={entry.sql}
              >
                {entry.sql}
              </div>
              <div className="mt-1 flex gap-3 text-xs text-muted-foreground">
                <span>{t("query.duration", { duration: entry.durationMs })}</span>
                <span>
                  {t("query.rowsAffected", { count: entry.rowCount })}
                </span>
                <span>{new Date(entry.executedAt).toLocaleTimeString()}</span>
              </div>
            </Button>
          ))
        )}
      </div>
    </div>
  );
}
