import { useState } from "react";

import { useTranslation } from "@/commons/locales/useTranslation";
import { useRecentStore } from "@/commons/stores/recent.store";
import { Button } from "@/components/ui/button";

import { BackupDialog } from "@/modules/backup/components/backup-dialog";
import { RestoreDialog } from "@/modules/backup/components/restore-dialog";
import { useBackupDatabase, useRestoreDatabase } from "@/modules/backup/queries/backup.queries";
import type { BackupFormat } from "@/modules/backup/types/backup.types";

import { ConnectionList } from "../components/connection-list";

export function ConnectionsPage() {
  const { t } = useTranslation();
  const openConnectionDialog = useRecentStore((s) => s.openConnectionDialog);
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
    <div className="flex h-full flex-col gap-4 overflow-y-auto p-5">
      <div className="flex items-center justify-between">
        <h1 className="text-lg font-semibold text-foreground">
          {t("connection.title")}
        </h1>
        <Button type="button" onClick={() => openConnectionDialog()}>
          {t("connection.new")}
        </Button>
      </div>

      <ConnectionList
        onEdit={(id) => openConnectionDialog(id)}
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
