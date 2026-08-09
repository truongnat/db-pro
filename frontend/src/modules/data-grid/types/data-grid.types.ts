import type { CellValue, ColumnMeta, Row } from "@/modules/query/types/query.types";

export type { CellValue, ColumnMeta, Row };

export type FilterOp = "eq" | "neq" | "lt" | "lte" | "gt" | "gte" | "like" | "isNull" | "isNotNull";

export interface GridFilter {
  column: string;
  op: FilterOp;
  value: CellValue;
}

export interface GridSort {
  column: string;
  direction: "asc" | "desc";
}

export interface FetchRowsRequest {
  schema: string;
  table: string;
  filters: GridFilter[];
  sorts: GridSort[];
  page: number;
  pageSize: number;
}

export interface FetchRowsResult {
  columns: ColumnMeta[];
  rows: Row[];
  totalCount: number;
  durationMs: number;
}

export interface MutateRowRequest {
  schema: string;
  table: string;
  columns: string[];
  values: CellValue[];
  pkColumns?: string[];
  pkValues?: CellValue[];
}

export interface MutateRowResult {
  affectedRows: number;
}

export interface TableOption {
  name: string;
  schema: string;
}

export const FILTER_OPS: { value: FilterOp; label: string }[] = [
  { value: "eq", label: "=" },
  { value: "neq", label: "!=" },
  { value: "lt", label: "<" },
  { value: "lte", label: "<=" },
  { value: "gt", label: ">" },
  { value: "gte", label: ">=" },
  { value: "like", label: "LIKE" },
  { value: "isNull", label: "IS NULL" },
  { value: "isNotNull", label: "IS NOT NULL" },
];

export const VALUELESS_OPS: FilterOp[] = ["isNull", "isNotNull"];
