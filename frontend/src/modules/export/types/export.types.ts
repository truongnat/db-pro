export type ExportFormat = "csv" | "json" | "excel";

export interface ExportResult {
  fileContent: string;
  fileName: string;
  mimeType: string;
  rowCount: number;
}
