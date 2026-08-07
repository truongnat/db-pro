import type { ExplainPlan, QueryResult } from "@/modules/query/types/query.types";

export type WorkspaceTabKind = "query" | "db-object";

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

export type ResultPanelTab = "results" | "explain" | "history" | "local-history";

export interface QueryTabData {
  sql: string;
  status: ExecutionStatus;
  error: string | null;
  result: QueryResult | null;
  explainPlan: ExplainPlan | null;
  sort: SortState;
  multiResults: QueryResult[] | null;
  multiResultIndex: number;
  activePanel: ResultPanelTab;
}

export type DbObjectSection = "data" | "structure" | "indexes" | "relations" | "ddl" | "triggers";

export interface DbObjectTabData {
  schema: string;
  objectName: string;
  objectType: "table" | "view" | "function" | "sequence" | "type";
  activeSection: DbObjectSection;
}

type WorkspaceTabDataMap = {
  query: QueryTabData;
  "db-object": DbObjectTabData;
};

export type WorkspaceTab = {
  [K in WorkspaceTabKind]: WorkspaceTabBase & { kind: K; data: WorkspaceTabDataMap[K] };
}[WorkspaceTabKind];

export interface PersistedWorkspaceState {
  tabs: WorkspaceTab[];
  activeTabId: string | null;
  recentlyClosed: WorkspaceTab[];
}
