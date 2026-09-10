export type BackupFormat = "plain" | "custom";

export interface BackupOptions {
  connectionId: string;
  outputPath: string;
  format: BackupFormat;
  schemas?: string[];
  tables?: string[];
}

export interface RestoreOptions {
  connectionId: string;
  inputPath: string;
  format: BackupFormat;
}

export interface BackupResult {
  outputPath: string;
  sizeBytes: number;
}
