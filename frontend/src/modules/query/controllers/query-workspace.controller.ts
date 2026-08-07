import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import type {
  ExecutionStatus,
  QueryTabData,
  SortState,
  WorkspaceTab,
} from "@/commons/types/workspace.types";
import type { ExplainPlan, QueryResult } from "@/modules/query/types/query.types";

type QueryWorkspaceTab = Omit<WorkspaceTab, "data"> & { data: QueryTabData };

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
