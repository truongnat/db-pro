import type { ExplainPlan, QueryResult } from "@/modules/query/types/query.types";

export type WorkspaceTabKind = "query" | "table-data" | "schema-object";

export type ExecutionStatus = "idle" | "running" | "success" | "error";

export interface SortState {
  column: string | null;
  direction: "asc" | "desc" | null;
}

export interface WorkspaceTabBase {
  id: string;
  kind: WorkspaceTabKind;
  title: string;
  connectionId: string | null;
  resourceKey: string;
  dirty: boolean;
  pinned: boolean;
  preview: boolean;
  order: number;
}

export interface QueryTabData {
  sql: string;
  status: ExecutionStatus;
  error: string | null;
  result: QueryResult | null;
  explainPlan: ExplainPlan | null;
  sort: SortState;
  multiResults: QueryResult[] | null;
  multiResultIndex: number;
}

export interface TableDataTabData {
  schema: string;
  table: string;
}

export interface SchemaObjectTabData {
  schema: string;
  objectName: string;
  objectType: "table" | "view" | "function" | "sequence" | "type";
}

type WorkspaceTabDataMap = {
  query: QueryTabData;
  "table-data": TableDataTabData;
  "schema-object": SchemaObjectTabData;
};

export type WorkspaceTab = {
  [K in WorkspaceTabKind]: WorkspaceTabBase & { kind: K; data: WorkspaceTabDataMap[K] };
}[WorkspaceTabKind];

export type ResultPanelTab = "results" | "explain" | "history" | "local-history";

export interface PersistedWorkspaceState {
  tabs: WorkspaceTab[];
  activeTabId: string | null;
  recentlyClosed: WorkspaceTab[];
}
