export type ExportFormat = "csv" | "json" | "excel" | "sql";

export type ExportScope = "all" | "selected";

export interface ExportOptions {
  format: ExportFormat;
  scope: ExportScope;
  /** CSV delimiter */
  delimiter: "," | ";" | "\t" | "|";
  /** Include column headers (CSV) */
  includeHeaders: boolean;
  /** How to represent NULL values */
  nullRepresentation: string;
  /** Table name for SQL INSERT export */
  tableName: string;
  /** Pretty-print JSON */
  prettyJson: boolean;
}

export const DEFAULT_EXPORT_OPTIONS: ExportOptions = {
  format: "csv",
  scope: "all",
  delimiter: ",",
  includeHeaders: true,
  nullRepresentation: "",
  tableName: "",
  prettyJson: true,
};

export interface ExportResult {
  fileContent: string;
  fileName: string;
  mimeType: string;
  rowCount: number;
}
