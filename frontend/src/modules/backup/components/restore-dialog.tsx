import { useState } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/button";

import type { BackupFormat } from "../types/backup.types";

interface RestoreDialogProps {
  open: boolean;
  connectionId: string;
  onClose: () => void;
  onRestore: (inputPath: string, format: BackupFormat) => void;
  isPending: boolean;
}

export function RestoreDialog({
  open,
  connectionId: _connectionId,
  onClose,
  onRestore,
  isPending,
}: RestoreDialogProps) {
  const { t } = useTranslation();
  const [inputPath, setInputPath] = useState("");
  const [format, setFormat] = useState<BackupFormat>("plain");

  if (!open) return null;

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!inputPath.trim()) return;
    onRestore(inputPath.trim(), format);
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center"
      style={{ backgroundColor: "rgba(0,0,0,0.4)" }}
    >
      <form
        onSubmit={handleSubmit}
        className="flex w-[28rem] flex-col gap-4 rounded-md border border-border bg-background p-6"
      >
        <h2 className="text-lg font-semibold text-foreground">
          {t("backup.restoreTitle")}
        </h2>

        <p className="text-xs text-destructive">
          {t("backup.restoreWarning")}
        </p>

        <label className="flex flex-col gap-1">
          <span className="text-sm text-muted-foreground">
            {t("backup.inputPath")}
          </span>
          <input
            type="text"
            value={inputPath}
            onChange={(e) => setInputPath(e.target.value)}
            placeholder="/path/to/backup.sql"
            className="rounded-sm border border-border bg-card px-3 py-2 text-sm text-foreground"
            autoFocus
          />
        </label>

        <label className="flex flex-col gap-1">
          <span className="text-sm text-muted-foreground">
            {t("backup.format")}
          </span>
          <select
            value={format}
            onChange={(e) => setFormat(e.target.value as BackupFormat)}
            className="rounded-sm border border-border bg-card px-3 py-2 text-sm text-foreground"
          >
            <option value="plain">{t("backup.formatPlain")}</option>
            <option value="custom">{t("backup.formatCustom")}</option>
          </select>
        </label>

        <div className="flex justify-end gap-2">
          <Button type="button" variant="outline" onClick={onClose}>
            {t("common.actions.cancel")}
          </Button>
          <Button
            type="submit"
            disabled={!inputPath.trim() || isPending}
          >
            {isPending ? t("backup.restoreInProgress") : t("backup.startRestore")}
          </Button>
        </div>
      </form>
    </div>
  );
}
