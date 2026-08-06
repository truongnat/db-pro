import { create } from "zustand";

import type { ExplainPlan, QueryResult } from "../types/query.types";

export type ExecutionStatus = "idle" | "running" | "success" | "error";

export interface SortState {
  column: string | null;
  direction: "asc" | "desc" | null;
}

export type ResultTab = "results" | "explain" | "history";

interface QueryModuleState {
  sql: string;
  status: ExecutionStatus;
  error: string | null;
  result: QueryResult | null;
  explainPlan: ExplainPlan | null;
  sort: SortState;
  activeTab: ResultTab;
  historySearch: string;

  setSql: (sql: string) => void;
  setStatus: (status: ExecutionStatus) => void;
  setError: (error: string | null) => void;
  setResult: (result: QueryResult | null) => void;
  setExplainPlan: (plan: ExplainPlan | null) => void;
  setSort: (sort: SortState) => void;
  setActiveTab: (tab: ResultTab) => void;
  setHistorySearch: (search: string) => void;
  reset: () => void;
}

const initialState = {
  sql: "",
  status: "idle" as ExecutionStatus,
  error: null,
  result: null,
  explainPlan: null,
  sort: { column: null, direction: null } as SortState,
  activeTab: "results" as ResultTab,
  historySearch: "",
};

export const useQueryModuleStore = create<QueryModuleState>()((set) => ({
  ...initialState,

  setSql: (sql) => set({ sql }),
  setStatus: (status) => set({ status }),
  setError: (error) => set({ error }),
  setResult: (result) => set({ result }),
  setExplainPlan: (plan) => set({ explainPlan: plan }),
  setSort: (sort) => set({ sort }),
  setActiveTab: (tab) => set({ activeTab: tab }),
  setHistorySearch: (search) => set({ historySearch: search }),
  reset: () => set(initialState),
}));
