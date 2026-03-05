export type DbEngine = "sqlite" | "postgres" | "mysql";

export interface ConnectionInput {
  id?: string;
  name: string;
  engine: DbEngine;
  path?: string;
  host?: string;
  port?: number;
  database?: string;
  username?: string;
  password?: string;
}

export interface ConnectionListItem {
  id: string;
  name: string;
  engine: DbEngine;
  target: string;
}

export interface QueryRequest {
  connectionId: string;
  sql: string;
  limit?: number;
  pageSize?: number;
  offset?: number;
  timeoutMs?: number;
  quickFilter?: string;
  filterColumns?: string[];
  sortColumn?: string;
  sortDirection?: "asc" | "desc";
}

export interface QueryResult {
  columns: string[];
  rows: string[][];
  affectedRows: number;
  executionMs: number;
  message: string;
  schemaChanged: boolean;
  isRowQuery: boolean;
  pageSize: number;
  pageOffset: number;
  hasMore: boolean;
}

export type NavigatorObjectKind = "table" | "view";

export interface NavigatorColumn {
  name: string;
  dataType: string;
  nullable: boolean;
}

export interface NavigatorObject {
  name: string;
  kind: NavigatorObjectKind;
  columns: NavigatorColumn[];
}

export interface NavigatorSchema {
  name: string;
  tables: NavigatorObject[];
  views: NavigatorObject[];
}

export interface NavigatorTree {
  schemas: NavigatorSchema[];
  warnings: string[];
}
