import type { ExplainPlan, QueryResult } from "@/modules/query/types/query.types";

export type WorkspaceTabKind = "query" | "db-object";

export type ExecutionStatus = "idle" | "running" | "success" | "error" | "cancelled";

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

export type ResultPanelTab =
  "results" | "explain" | "messages" | "history" | "local-history" | "snippets";

export interface QueryContext {
  database: string | null;
  schema: string | null;
}

/**
 * Client-side timing breakdown for a query execution.
 * - serverMs: time reported by the backend (DB execution)
 * - totalMs: wall-clock time from submit to result received
 * - fetchMs: totalMs - serverMs (network + serialization overhead)
 * - renderMs: time to first render of the result grid
 */
export interface QueryTiming {
  serverMs: number;
  totalMs: number;
  fetchMs: number;
  renderMs: number;
}

export interface QueryTabData {
  context: QueryContext;
  sql: string;
  status: ExecutionStatus;
  error: string | null;
  result: QueryResult | null;
  explainPlan: ExplainPlan | null;
  sort: SortState;
  multiResults: QueryResult[] | null;
  multiResultIndex: number;
  activePanel: ResultPanelTab;
  timing: QueryTiming | null;
  /** Timestamp (Date.now()) when the current execution started. */
  executionStartedAt: number | null;
  /** Unique execution ID for the current in-flight query. Null when idle. */
  activeExecutionId: string | null;
}

export type DbObjectSection = "data" | "columns" | "indexes" | "relations" | "ddl" | "triggers" | "diagram";

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
  workspaceVersion: number;
  tabs: WorkspaceTab[];
  activeTabId: string | null;
  recentlyClosed: WorkspaceTab[];
}
