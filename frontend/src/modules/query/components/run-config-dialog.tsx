import { useCallback, useEffect, useState } from "react";

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
        className="w-[480px] rounded-[var(--radius-md)] p-4 shadow-lg"
        style={{
          backgroundColor: "var(--color-bg-secondary, #1e293b)",
          border: "1px solid var(--color-border)",
        }}
        onClick={(e) => e.stopPropagation()}
      >
        <h3
          className="mb-3 text-sm font-semibold"
          style={{ color: "var(--color-text)" }}
        >
          {t("query.newRunConfig")}
        </h3>

        <div className="space-y-3">
          <div>
            <label
              className="mb-1 block text-xs"
              style={{ color: "var(--color-text-secondary)" }}
            >
              {t("query.configName")}
            </label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="w-full rounded-[var(--radius-sm)] border px-2 py-1.5 text-sm outline-none focus:border-[var(--color-primary,#3b82f6)]"
              style={{
                borderColor: "var(--color-border)",
                backgroundColor: "var(--color-bg)",
                color: "var(--color-text)",
              }}
            />
          </div>

          <div>
            <label
              className="mb-1 block text-xs"
              style={{ color: "var(--color-text-secondary)" }}
            >
              SQL
            </label>
            <textarea
              value={sql}
              onChange={(e) => setSql(e.target.value)}
              rows={4}
              className="w-full resize-y rounded-[var(--radius-sm)] border px-2 py-1.5 font-mono text-xs outline-none focus:border-[var(--color-primary,#3b82f6)]"
              style={{
                borderColor: "var(--color-border)",
                backgroundColor: "var(--color-bg)",
                color: "var(--color-text)",
              }}
            />
          </div>

          <div className="flex gap-3">
            <div className="flex-1">
              <label
                className="mb-1 block text-xs"
                style={{ color: "var(--color-text-secondary)" }}
              >
                {t("query.timeoutMs")}
              </label>
              <input
                type="number"
                value={timeoutMs}
                onChange={(e) => setTimeoutMs(Number(e.target.value))}
                min={1000}
                step={1000}
                className="w-full rounded-[var(--radius-sm)] border px-2 py-1.5 text-sm outline-none focus:border-[var(--color-primary,#3b82f6)]"
                style={{
                  borderColor: "var(--color-border)",
                  backgroundColor: "var(--color-bg)",
                  color: "var(--color-text)",
                }}
              />
            </div>
            <div className="flex-1">
              <label
                className="mb-1 block text-xs"
                style={{ color: "var(--color-text-secondary)" }}
              >
                {t("query.maxRows")}
              </label>
              <input
                type="number"
                value={maxRows}
                onChange={(e) => setMaxRows(Number(e.target.value))}
                min={1}
                className="w-full rounded-[var(--radius-sm)] border px-2 py-1.5 text-sm outline-none focus:border-[var(--color-primary,#3b82f6)]"
                style={{
                  borderColor: "var(--color-border)",
                  backgroundColor: "var(--color-bg)",
                  color: "var(--color-text)",
                }}
              />
            </div>
          </div>
        </div>

        <div className="mt-4 flex justify-end gap-2">
          <button
            type="button"
            onClick={onClose}
            className="rounded-[var(--radius-sm)] px-3 py-1.5 text-sm transition-colors hover:opacity-80"
            style={{ color: "var(--color-text-secondary)" }}
          >
            {t("common.actions.cancel")}
          </button>
          <button
            type="button"
            onClick={handleSave}
            disabled={!name.trim() || !sql.trim() || saveMutation.isPending}
            className="rounded-[var(--radius-sm)] px-3 py-1.5 text-sm text-white transition-colors hover:opacity-90 disabled:opacity-50"
            style={{ backgroundColor: "var(--color-primary, #3b82f6)" }}
          >
            {t("common.actions.save")}
          </button>
        </div>
      </div>
    </div>
  );
}
