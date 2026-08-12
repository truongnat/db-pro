import { useCallback } from "react";

import { Button } from "@/components/ui/button";
import { useTranslation } from "@/commons/locales/useTranslation";

import { useDeleteRunConfig, useListRunConfigs } from "../queries/query.queries";
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
        <span className="font-medium text-foreground">{t("query.runConfigs")}</span>
        <Button
          type="button"
          variant="ghost"
          size="sm"
          onClick={onNew}
          className="rounded-sm px-2 py-0.5 text-xs text-primary"
        >
          + {t("query.newRunConfig")}
        </Button>
      </div>

      {configs.map((config) => (
        <div
          key={config.id}
          className="group flex items-center justify-between rounded-sm px-2 py-1 transition-colors hover:bg-background"
        >
          <Button
            type="button"
            variant="ghost"
            className="flex-1 justify-start truncate text-left text-xs text-foreground"
            onClick={() => onSelect(config)}
            title={config.name}
          >
            {config.name}
          </Button>
          <Button
            type="button"
            variant="ghost"
            size="sm"
            className="ml-1 text-destructive opacity-0 transition-opacity group-hover:opacity-100"
            onClick={() => handleDelete(config.id)}
            title={t("common.actions.delete")}
          >
            ×
          </Button>
        </div>
      ))}

      {configs.length === 0 && (
        <div className="py-4 text-center text-xs italic text-[var(--text-secondary)]">
          {t("query.noRunConfigs")}
        </div>
      )}
    </div>
  );
}
