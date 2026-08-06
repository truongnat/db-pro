import { useCallback } from "react";

import { useTranslation } from "@/commons/locales/useTranslation";

import {
  useDeleteRunConfig,
  useListRunConfigs,
} from "../queries/query.queries";
import type { RunConfig } from "../types/query.types";

interface RunConfigListProps {
  connectionId: string;
  onSelect: (config: RunConfig) => void;
  onNew: () => void;
}

export function RunConfigList({ connectionId, onSelect, onNew }: RunConfigListProps) {
  const { t } = useTranslation();
  const configsQuery = useListRunConfigs(connectionId);
  const deleteMutation = useDeleteRunConfig();

  const configs: RunConfig[] = configsQuery.data ?? [];

  const handleDelete = useCallback(
    (id: string) => {
      deleteMutation.mutate({ id, connectionId });
    },
    [connectionId, deleteMutation],
  );

  return (
    <div className="flex h-full flex-col overflow-auto p-2 text-sm">
      <div className="mb-2 flex items-center justify-between">
        <span className="font-medium" style={{ color: "var(--color-text)" }}>
          {t("query.runConfigs")}
        </span>
        <button
          type="button"
          onClick={onNew}
          className="rounded-[var(--radius-sm)] px-2 py-0.5 text-xs transition-colors hover:bg-[var(--color-bg)]"
          style={{ color: "var(--color-primary, #3b82f6)" }}
        >
          + {t("query.newRunConfig")}
        </button>
      </div>

      {configs.map((config) => (
        <div
          key={config.id}
          className="group flex items-center justify-between rounded-[var(--radius-sm)] px-2 py-1 transition-colors hover:bg-[var(--color-bg)]"
        >
          <button
            type="button"
            className="flex-1 truncate text-left text-xs"
            style={{ color: "var(--color-text)" }}
            onClick={() => onSelect(config)}
            title={config.name}
          >
            {config.name}
          </button>
          <button
            type="button"
            className="ml-1 opacity-0 transition-opacity group-hover:opacity-100"
            style={{ color: "var(--color-error, #ef4444)" }}
            onClick={() => handleDelete(config.id)}
            title={t("common.actions.delete")}
          >
            ×
          </button>
        </div>
      ))}

      {configs.length === 0 && (
        <div
          className="py-4 text-center text-xs italic"
          style={{ color: "var(--color-text-secondary)" }}
        >
          {t("query.noRunConfigs")}
        </div>
      )}
    </div>
  );
}
