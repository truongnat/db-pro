import { useCallback, useEffect, useState } from "react";

import { Button } from "@/components/ui/button";
import { useTranslation } from "@/commons/locales/useTranslation";

import { useSaveRunConfig } from "../queries/query.queries";

interface RunConfigDialogProps {
  open: boolean;
  onClose: () => void;
  connectionId: string;
  defaultName?: string;
  defaultSql?: string;
}

export function RunConfigDialog({
  open,
  onClose,
  connectionId,
  defaultName = "",
  defaultSql = "",
}: RunConfigDialogProps) {
  const { t } = useTranslation();
  const saveMutation = useSaveRunConfig();

  const [name, setName] = useState(defaultName);
  const [sql, setSql] = useState(defaultSql);
  const [timeoutMs, setTimeoutMs] = useState(30000);
  const [maxRows, setMaxRows] = useState(1000);

  useEffect(() => {
    if (open) {
      setName(defaultName);
      setSql(defaultSql);
      setTimeoutMs(30000);
      setMaxRows(1000);
    }
  }, [open, defaultName, defaultSql]);

  const handleSave = useCallback(() => {
    if (!name.trim() || !sql.trim()) return;
    saveMutation.mutate(
      {
        connectionId,
        name: name.trim(),
        sql: sql.trim(),
        timeoutMs,
        maxRows,
      },
      { onSuccess: () => onClose() },
    );
  }, [connectionId, name, sql, timeoutMs, maxRows, saveMutation, onClose]);

  if (!open) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center"
      style={{ backgroundColor: "rgba(0,0,0,0.4)" }}
      onClick={onClose}
    >
      <div
        className="w-[480px] rounded-md border border-border bg-muted p-4 shadow-lg"
        onClick={(e) => e.stopPropagation()}
      >
        <h3 className="mb-3 text-sm font-semibold text-foreground">
          {t("query.newRunConfig")}
        </h3>

        <div className="space-y-3">
          <div>
            <label className="mb-1 block text-xs text-muted-foreground">
              {t("query.configName")}
            </label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="w-full rounded-sm border border-border bg-background px-2 py-1.5 text-sm text-foreground outline-none focus:border-primary"
            />
          </div>

          <div>
            <label className="mb-1 block text-xs text-muted-foreground">
              SQL
            </label>
            <textarea
              value={sql}
              onChange={(e) => setSql(e.target.value)}
              rows={4}
              className="w-full resize-y rounded-sm border border-border bg-background px-2 py-1.5 font-mono text-xs text-foreground outline-none focus:border-primary"
            />
          </div>

          <div className="flex gap-3">
            <div className="flex-1">
              <label className="mb-1 block text-xs text-muted-foreground">
                {t("query.timeoutMs")}
              </label>
              <input
                type="number"
                value={timeoutMs}
                onChange={(e) => setTimeoutMs(Number(e.target.value))}
                min={1000}
                step={1000}
                className="w-full rounded-sm border border-border bg-background px-2 py-1.5 text-sm text-foreground outline-none focus:border-primary"
              />
            </div>
            <div className="flex-1">
              <label className="mb-1 block text-xs text-muted-foreground">
                {t("query.maxRows")}
              </label>
              <input
                type="number"
                value={maxRows}
                onChange={(e) => setMaxRows(Number(e.target.value))}
                min={1}
                className="w-full rounded-sm border border-border bg-background px-2 py-1.5 text-sm text-foreground outline-none focus:border-primary"
              />
            </div>
          </div>
        </div>

        <div className="mt-4 flex justify-end gap-2">
          <Button
            type="button"
            variant="ghost"
            className="rounded-sm px-3 py-1.5 text-sm text-muted-foreground"
            onClick={onClose}
          >
            {t("common.actions.cancel")}
          </Button>
          <Button
            type="button"
            onClick={handleSave}
            disabled={!name.trim() || !sql.trim() || saveMutation.isPending}
            className="rounded-sm px-3 py-1.5 text-sm"
          >
            {t("common.actions.save")}
          </Button>
        </div>
      </div>
    </div>
  );
}
