import { createQueryTab, type CreateQueryTabOptions } from "@/commons/factories/tab-factories";
import { useConnectionStore } from "@/commons/stores/connection.store";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import type {
  ExecutionStatus,
  QueryContext,
  QueryTabData,
  QueryTiming,
  ResultPanelTab,
  SortState,
  WorkspaceTab,
} from "@/commons/types/workspace.types";
import type { ExplainPlan, QueryResult } from "@/modules/query/types/query.types";

type QueryWorkspaceTab = Omit<WorkspaceTab, "data"> & { data: QueryTabData };

function getKnownConnections(): { id: string; database: string }[] {
  return useConnectionStore.getState().connections;
}

export function getActiveQueryTab(): QueryWorkspaceTab | undefined {
  const { tabs, activeTabId } = useWorkspaceStore.getState();
  return tabs.find(
    (t) => t.id === activeTabId && t.kind === "query",
  ) as QueryWorkspaceTab | undefined;
}

export function getQueryTabData(tabId: string): QueryTabData | undefined {
  const { tabs } = useWorkspaceStore.getState();
  const tab = tabs.find((t) => t.id === tabId && t.kind === "query");
  return tab ? (tab.data as QueryTabData) : undefined;
}

export function getQueryContext(tabId: string): QueryContext | undefined {
  return getQueryTabData(tabId)?.context;
}

export function buildQueryContext(
  connections: { id: string; database: string }[],
  connectionId: string | null,
  schema: string | null,
): QueryContext {
  const conn = connections.find((c) => c.id === connectionId);
  return { database: conn?.database ?? null, schema };
}

export function createExplorerQueryContext(
  connections: { id: string; database: string }[],
  explorerConnectionId: string | null,
): QueryContext {
  return buildQueryContext(connections, explorerConnectionId, null);
}

export function createQueryTabFromExplorerContext(
  explorerConnectionId: string | null,
  options?: CreateQueryTabOptions,
): (WorkspaceTab & { kind: "query" }) | undefined {
  if (!explorerConnectionId) return undefined;
  const context = createExplorerQueryContext(
    getKnownConnections(),
    explorerConnectionId,
  );
  return createQueryTab(explorerConnectionId, { ...options, context });
}

export function createQueryTabForObject(
  connectionId: string,
  schema: string,
  options?: CreateQueryTabOptions,
): WorkspaceTab & { kind: "query" } {
  const context = buildQueryContext(getKnownConnections(), connectionId, schema);
  return createQueryTab(connectionId, { ...options, context });
}

function updateData(
  tabId: string,
  updater: (data: QueryTabData) => QueryTabData,
): void {
  useWorkspaceStore.getState().updateTabData(tabId, updater);
}

export function setTabSql(tabId: string, sql: string): void {
  const store = useWorkspaceStore.getState();
  const tab = store.tabs.find((t) => t.id === tabId);
  updateData(tabId, (data) => ({ ...data, sql }));
  if (tab && !tab.dirty) {
    store.setTabDirty(tabId, true);
  }
}

export function setTabStatus(tabId: string, status: ExecutionStatus): void {
  updateData(tabId, (data) => ({ ...data, status }));
}

export function setTabError(tabId: string, error: string | null): void {
  updateData(tabId, (data) => ({ ...data, error }));
}

export function setTabResult(tabId: string, result: QueryResult | null): void {
  updateData(tabId, (data) => ({ ...data, result }));
}

export function setTabExplainPlan(tabId: string, plan: ExplainPlan | null): void {
  updateData(tabId, (data) => ({ ...data, explainPlan: plan }));
}

export function setTabSort(tabId: string, sort: SortState): void {
  updateData(tabId, (data) => ({ ...data, sort }));
}

export function setTabMultiResults(
  tabId: string,
  results: QueryResult[] | null,
): void {
  updateData(tabId, (data) => ({ ...data, multiResults: results, multiResultIndex: 0 }));
}

export function setTabMultiResultIndex(tabId: string, index: number): void {
  updateData(tabId, (data) => ({ ...data, multiResultIndex: index }));
}

export function setTabActivePanel(tabId: string, panel: ResultPanelTab): void {
  updateData(tabId, (data) => ({ ...data, activePanel: panel }));
}

export function setQueryTabConnection(
  tabId: string,
  connectionId: string,
  context: QueryContext,
): void {
  useWorkspaceStore.getState().setQueryTabConnection(tabId, connectionId, context);
}

export function setQueryTabSchema(tabId: string, schema: string | null): void {
  updateData(tabId, (data) => ({ ...data, context: { ...data.context, schema } }));
}

export function setTabTiming(tabId: string, timing: QueryTiming | null): void {
  updateData(tabId, (data) => ({ ...data, timing }));
}

export function setTabExecutionStartedAt(tabId: string, at: number | null): void {
  updateData(tabId, (data) => ({ ...data, executionStartedAt: at }));
}
