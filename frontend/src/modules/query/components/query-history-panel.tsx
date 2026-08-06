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
      <div className="border-b p-2" style={{ borderColor: "var(--color-border)" }}>
        <input
          type="text"
          value={search}
          onChange={(e) => onSearchChange(e.target.value)}
          placeholder={t("query.searchHistory")}
          className="w-full rounded-[var(--radius-sm)] border px-3 py-1.5 text-sm outline-none transition-colors focus:border-[var(--color-primary,#3b82f6)]"
          style={{
            borderColor: "var(--color-border)",
            backgroundColor: "var(--color-bg)",
            color: "var(--color-text)",
          }}
        />
      </div>

      <div className="flex-1 overflow-auto">
        {isLoading ? (
          <div className="flex items-center justify-center py-8">
            <p style={{ color: "var(--color-text-secondary)" }}>
              {t("common.states.loading")}
            </p>
          </div>
        ) : filtered.length === 0 ? (
          <div className="flex items-center justify-center py-8">
            <p style={{ color: "var(--color-text-secondary)" }}>
              {t("common.states.empty")}
            </p>
          </div>
        ) : (
          filtered.map((entry) => (
            <button
              key={entry.id}
              className="w-full border-b px-3 py-2 text-left transition-colors hover:bg-[var(--color-surface)]"
              style={{ borderColor: "var(--color-border)" }}
              onClick={() => onSelectEntry(entry.sql)}
            >
              <div
                className="truncate text-sm font-mono"
                style={{ color: "var(--color-text)" }}
                title={entry.sql}
              >
                {entry.sql}
              </div>
              <div
                className="mt-1 flex gap-3 text-xs"
                style={{ color: "var(--color-text-secondary)" }}
              >
                <span>{t("query.duration", { duration: entry.durationMs })}</span>
                <span>
                  {t("query.rowsAffected", { count: entry.rowCount })}
                </span>
                <span>{new Date(entry.executedAt).toLocaleTimeString()}</span>
              </div>
            </button>
          ))
        )}
      </div>
    </div>
  );
}
