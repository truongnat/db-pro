export type CellValue =
  | { type: "null" }
  | { type: "bool"; value: boolean }
  | { type: "int64"; value: number }
  | { type: "float64"; value: number }
  | { type: "text"; value: string }
  | { type: "bytes"; value: number[] }
  | { type: "uuid"; value: string }
  | { type: "datetime"; value: string }
  | { type: "json"; value: unknown };

export interface ColumnMeta {
  name: string;
  dataType: string;
  nullable: boolean;
}

export type Row = CellValue[];

export interface QueryResult {
  columns: ColumnMeta[];
  rows: Row[];
  rowCount: number;
  durationMs: number;
}

export interface QueryHistoryEntry {
  id: string;
  connectionId: string;
  sql: string;
  executedAt: string;
  durationMs: number;
  rowCount: number;
}

export interface SavedQuery {
  id: string;
  connectionId: string;
  name: string;
  sql: string;
  folder?: string;
  createdAt: string;
}

export type ExplainPlan = Record<string, unknown>;

export function renderCellValue(cell: CellValue): string {
  switch (cell.type) {
    case "null":
      return "NULL";
    case "bool":
      return cell.value ? "true" : "false";
    case "bytes":
      return `<binary (${cell.value.length} bytes)>`;
    case "json":
      return JSON.stringify(cell.value, null, 2);
    case "int64":
    case "float64":
    case "text":
    case "uuid":
    case "datetime":
      return String(cell.value);
  }
}
