import { apiInvoke } from "@/commons/utils/api";

import type { BackupOptions, BackupResult, RestoreOptions } from "../types/backup.types";

export class BackupService {
  async backup(options: BackupOptions): Promise<BackupResult> {
    return apiInvoke<BackupResult>("backup_database", {
      req: {
        connectionId: options.connectionId,
        outputPath: options.outputPath,
        format: options.format,
        schemas: options.schemas,
        tables: options.tables,
      },
    });
  }

  async restore(options: RestoreOptions): Promise<void> {
    return apiInvoke<void>("restore_database", {
      req: {
        connectionId: options.connectionId,
        inputPath: options.inputPath,
        format: options.format,
      },
    });
  }
}

export function createBackupService(): BackupService {
  return new BackupService();
}
