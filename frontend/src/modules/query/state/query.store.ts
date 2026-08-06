import { create } from "zustand";

import type { ExplainPlan, QueryResult } from "../types/query.types";

export type ExecutionStatus = "idle" | "running" | "success" | "error";

export interface SortState {
  column: string | null;
  direction: "asc" | "desc" | null;
}

export type ResultTab = "results" | "explain" | "history" | "local-history";

let tabCounter = 0;

export interface QueryTab {
  id: string;
  title: string;
  sql: string;
  status: ExecutionStatus;
  error: string | null;
  result: QueryResult | null;
  explainPlan: ExplainPlan | null;
  sort: SortState;
  multiResults: QueryResult[] | null;
  multiResultIndex: number;
}

function createTab(title?: string): QueryTab {
  tabCounter += 1;
  return {
    id: `tab-${tabCounter}`,
    title: title ?? `Query ${tabCounter}`,
    sql: "",
    status: "idle",
    error: null,
    result: null,
    explainPlan: null,
    sort: { column: null, direction: null },
    multiResults: null,
    multiResultIndex: 0,
  };
}

const defaultTab = createTab("Query 1");

interface QueryModuleState {
  tabs: QueryTab[];
  activeTabId: string;
  activeTab: ResultTab;
  historySearch: string;

  addTab: () => void;
  closeTab: (id: string) => void;
  setActiveTabId: (id: string) => void;
  setSql: (sql: string) => void;
  setStatus: (status: ExecutionStatus) => void;
  setError: (error: string | null) => void;
  setResult: (result: QueryResult | null) => void;
  setExplainPlan: (plan: ExplainPlan | null) => void;
  setSort: (sort: SortState) => void;
  setMultiResults: (results: QueryResult[] | null) => void;
  setMultiResultIndex: (index: number) => void;
  setActiveTab: (tab: ResultTab) => void;
  setHistorySearch: (search: string) => void;
  restoreTabs: (tabs: QueryTab[], activeTabId: string) => void;
  reset: () => void;
}

function updateActiveTab(
  tabs: QueryTab[],
  activeTabId: string,
  updater: (tab: QueryTab) => QueryTab,
): QueryTab[] {
  return tabs.map((t) => (t.id === activeTabId ? updater(t) : t));
}

export const useQueryModuleStore = create<QueryModuleState>()((set) => ({
  tabs: [defaultTab],
  activeTabId: defaultTab.id,
  activeTab: "results",
  historySearch: "",

  addTab: () =>
    set((state) => {
      const tab = createTab();
      return {
        tabs: [...state.tabs, tab],
        activeTabId: tab.id,
        activeTab: "results",
      };
    }),

  closeTab: (id: string) =>
    set((state) => {
      if (state.tabs.length <= 1) return state;
      const idx = state.tabs.findIndex((t) => t.id === id);
      const newTabs = state.tabs.filter((t) => t.id !== id);
      let newActiveId = state.activeTabId;
      if (id === state.activeTabId) {
        const nextIdx = Math.min(idx, newTabs.length - 1);
        newActiveId = newTabs[nextIdx].id;
      }
      return { tabs: newTabs, activeTabId: newActiveId };
    }),

  setActiveTabId: (id: string) => set({ activeTabId: id, activeTab: "results" }),

  setSql: (sql: string) =>
    set((state) => ({
      tabs: updateActiveTab(state.tabs, state.activeTabId, (t) => ({ ...t, sql })),
    })),

  setStatus: (status: ExecutionStatus) =>
    set((state) => ({
      tabs: updateActiveTab(state.tabs, state.activeTabId, (t) => ({ ...t, status })),
    })),

  setError: (error: string | null) =>
    set((state) => ({
      tabs: updateActiveTab(state.tabs, state.activeTabId, (t) => ({ ...t, error })),
    })),

  setResult: (result: QueryResult | null) =>
    set((state) => ({
      tabs: updateActiveTab(state.tabs, state.activeTabId, (t) => ({ ...t, result })),
    })),

  setExplainPlan: (explainPlan: ExplainPlan | null) =>
    set((state) => ({
      tabs: updateActiveTab(state.tabs, state.activeTabId, (t) => ({ ...t, explainPlan })),
    })),

  setSort: (sort: SortState) =>
    set((state) => ({
      tabs: updateActiveTab(state.tabs, state.activeTabId, (t) => ({ ...t, sort })),
    })),

  setMultiResults: (multiResults: QueryResult[] | null) =>
    set((state) => ({
      tabs: updateActiveTab(state.tabs, state.activeTabId, (t) => ({ ...t, multiResults, multiResultIndex: 0 })),
    })),

  setMultiResultIndex: (multiResultIndex: number) =>
    set((state) => ({
      tabs: updateActiveTab(state.tabs, state.activeTabId, (t) => ({ ...t, multiResultIndex })),
    })),

  setActiveTab: (activeTab: ResultTab) => set({ activeTab }),
  setHistorySearch: (historySearch: string) => set({ historySearch }),

  restoreTabs: (tabs: QueryTab[], activeTabId: string) =>
    set({
      tabs,
      activeTabId: tabs.some((t) => t.id === activeTabId) ? activeTabId : tabs[0]?.id ?? activeTabId,
    }),

  reset: () => {
    tabCounter = 0;
    const tab = createTab("Query 1");
    set({
      tabs: [tab],
      activeTabId: tab.id,
      activeTab: "results",
      historySearch: "",
    });
  },
}));
