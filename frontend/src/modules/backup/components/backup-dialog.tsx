import { useState } from "react";
import { useTranslation } from "react-i18next";

import type { BackupFormat } from "../types/backup.types";

interface BackupDialogProps {
  open: boolean;
  connectionId: string;
  onClose: () => void;
  onBackup: (outputPath: string, format: BackupFormat) => void;
  isPending: boolean;
}

export function BackupDialog({
  open,
  connectionId: _connectionId,
  onClose,
  onBackup,
  isPending,
}: BackupDialogProps) {
  const { t } = useTranslation();
  const [outputPath, setOutputPath] = useState("");
  const [format, setFormat] = useState<BackupFormat>("plain");

  if (!open) return null;

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!outputPath.trim()) return;
    onBackup(outputPath.trim(), format);
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center"
      style={{ backgroundColor: "rgba(0,0,0,0.4)" }}
    >
      <form
        onSubmit={handleSubmit}
        className="flex w-[28rem] flex-col gap-4 rounded-[var(--radius-md)] border p-6"
        style={{
          backgroundColor: "var(--color-bg)",
          borderColor: "var(--color-border)",
        }}
      >
        <h2
          className="text-lg font-semibold"
          style={{ color: "var(--color-text)" }}
        >
          {t("backup.title")}
        </h2>

        <label className="flex flex-col gap-1">
          <span
            className="text-sm"
            style={{ color: "var(--color-text-secondary)" }}
          >
            {t("backup.outputPath")}
          </span>
          <input
            type="text"
            value={outputPath}
            onChange={(e) => setOutputPath(e.target.value)}
            placeholder="/path/to/backup.sql"
            className="rounded-[var(--radius-sm)] border px-3 py-2 text-sm"
            style={{
              backgroundColor: "var(--color-surface)",
              borderColor: "var(--color-border)",
              color: "var(--color-text)",
            }}
            autoFocus
          />
        </label>

        <label className="flex flex-col gap-1">
          <span
            className="text-sm"
            style={{ color: "var(--color-text-secondary)" }}
          >
            {t("backup.format")}
          </span>
          <select
            value={format}
            onChange={(e) => setFormat(e.target.value as BackupFormat)}
            className="rounded-[var(--radius-sm)] border px-3 py-2 text-sm"
            style={{
              backgroundColor: "var(--color-surface)",
              borderColor: "var(--color-border)",
              color: "var(--color-text)",
            }}
          >
            <option value="plain">{t("backup.formatPlain")}</option>
            <option value="custom">{t("backup.formatCustom")}</option>
          </select>
        </label>

        <div className="flex justify-end gap-2">
          <button
            type="button"
            onClick={onClose}
            className="rounded-[var(--radius-sm)] border px-4 py-2 text-sm transition-colors hover:bg-[var(--color-bg)]"
            style={{
              borderColor: "var(--color-border)",
              color: "var(--color-text)",
            }}
          >
            {t("common.actions.cancel")}
          </button>
          <button
            type="submit"
            disabled={!outputPath.trim() || isPending}
            className="rounded-[var(--radius-sm)] px-4 py-2 text-sm text-white transition-colors disabled:opacity-50"
            style={{ backgroundColor: "var(--color-primary)" }}
          >
            {isPending ? t("backup.backupInProgress") : t("backup.startBackup")}
          </button>
        </div>
      </form>
    </div>
  );
}
