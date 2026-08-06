import { useState } from "react";
import { useNavigate } from "@tanstack/react-router";

import { useTranslation } from "@/commons/locales/useTranslation";

import { BackupDialog } from "@/modules/backup/components/backup-dialog";
import { RestoreDialog } from "@/modules/backup/components/restore-dialog";
import { useBackupDatabase, useRestoreDatabase } from "@/modules/backup/queries/backup.queries";
import type { BackupFormat } from "@/modules/backup/types/backup.types";

import { ConnectionList } from "../components/connection-list";

export function ConnectionsPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [backupConnId, setBackupConnId] = useState<string | null>(null);
  const [restoreConnId, setRestoreConnId] = useState<string | null>(null);

  const backupMutation = useBackupDatabase();
  const restoreMutation = useRestoreDatabase();

  const handleBackup = (outputPath: string, format: BackupFormat) => {
    if (!backupConnId) return;
    backupMutation.mutate(
      { connectionId: backupConnId, outputPath, format },
      {
        onSuccess: (result) => {
          alert(t("backup.backupSuccess", { size: result.sizeBytes }));
          setBackupConnId(null);
        },
      },
    );
  };

  const handleRestore = (inputPath: string, format: BackupFormat) => {
    if (!restoreConnId) return;
    restoreMutation.mutate(
      { connectionId: restoreConnId, inputPath, format },
      {
        onSuccess: () => {
          alert(t("backup.restoreSuccess"));
          setRestoreConnId(null);
        },
      },
    );
  };

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center justify-between">
        <h1 className="text-xl font-semibold" style={{ color: "var(--color-text)" }}>
          {t("connection.title")}
        </h1>
        <button
          className="rounded-[var(--radius-sm)] px-4 py-2 text-sm text-white transition-colors hover:opacity-90"
          style={{ backgroundColor: "var(--color-primary,#3b82f6)" }}
          onClick={() => navigate({ to: "/connection-editor" })}
        >
          {t("connection.new")}
        </button>
      </div>

      <ConnectionList
        onEdit={(id) => navigate({ to: "/connection-editor", search: { id } })}
        onBackup={(id) => setBackupConnId(id)}
        onRestore={(id) => setRestoreConnId(id)}
      />

      <BackupDialog
        open={!!backupConnId}
        connectionId={backupConnId ?? ""}
        onClose={() => setBackupConnId(null)}
        onBackup={handleBackup}
        isPending={backupMutation.isPending}
      />

      <RestoreDialog
        open={!!restoreConnId}
        connectionId={restoreConnId ?? ""}
        onClose={() => setRestoreConnId(null)}
        onRestore={handleRestore}
        isPending={restoreMutation.isPending}
      />
    </div>
  );
}
