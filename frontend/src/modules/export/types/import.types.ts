export type ImportFormat = "csv" | "json";

export interface ImportColumnMapping {
  /** Index in the source file columns */
  sourceIndex: number;
  /** Source column name (from file header or JSON key) */
  sourceName: string;
  /** Target database column name (user-mapped) */
  targetColumn: string | null;
}

export interface ImportPreview {
  format: ImportFormat;
  sourceColumns: string[];
  /** First N rows for preview */
  sampleRows: Record<string, string>[];
  totalRowCount: number;
  mappings: ImportColumnMapping[];
}

export interface ImportValidationError {
  row: number;
  column: string;
  message: string;
}

export interface ImportResult {
  successCount: number;
  errorCount: number;
  errors: ImportValidationError[];
}
