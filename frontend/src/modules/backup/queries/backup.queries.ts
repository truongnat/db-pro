import { useMutation } from "@tanstack/react-query";

import { container } from "@/app/app.module";
import { SERVICE_NAMES, type IBackupService } from "@/commons/di/registry";

import type { BackupOptions, BackupResult, RestoreOptions } from "../types/backup.types";

function getBackupService() {
  return container.resolve<IBackupService>(SERVICE_NAMES.BACKUP_SERVICE);
}

export function useBackupDatabase() {
  return useMutation({
    mutationFn: (options: BackupOptions) =>
      getBackupService().backup(options) as Promise<BackupResult>,
  });
}

export function useRestoreDatabase() {
  return useMutation({
    mutationFn: (options: RestoreOptions) => getBackupService().restore(options),
  });
}
