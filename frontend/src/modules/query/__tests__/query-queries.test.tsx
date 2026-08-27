import { describe, expect, it, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactNode } from "react";

const {
  mockExecute,
  mockCancel,
  mockExecuteMulti,
  mockExplain,
  mockGetHistory,
  mockSaveQuery,
  mockListSaved,
  mockDeleteSavedQuery,
  mockListFolders,
  mockCreateFolder,
  mockDeleteFolder,
  mockListRunConfigs,
  mockSaveRunConfig,
  mockDeleteRunConfig,
  mockRenameSavedQuery,
  mockDuplicateSavedQuery,
  mockRuntimeCancelQuery,
  mockRuntimeExplainQuery,
} = vi.hoisted(() => ({
  mockExecute: vi.fn(),
  mockCancel: vi.fn(),
  mockExecuteMulti: vi.fn(),
  mockExplain: vi.fn(),
  mockGetHistory: vi.fn(),
  mockSaveQuery: vi.fn(),
  mockListSaved: vi.fn(),
  mockDeleteSavedQuery: vi.fn(),
  mockListFolders: vi.fn(),
  mockCreateFolder: vi.fn(),
  mockDeleteFolder: vi.fn(),
  mockListRunConfigs: vi.fn(),
  mockSaveRunConfig: vi.fn(),
  mockDeleteRunConfig: vi.fn(),
  mockRenameSavedQuery: vi.fn(),
  mockDuplicateSavedQuery: vi.fn(),
  mockRuntimeCancelQuery: vi.fn(),
  mockRuntimeExplainQuery: vi.fn(),
}));

vi.mock("@/app/app.module", () => ({
  container: {
    resolve: vi.fn(() => ({
      execute: mockExecute,
      cancel: mockCancel,
      executeMulti: mockExecuteMulti,
      explain: mockExplain,
      getHistory: mockGetHistory,
      saveQuery: mockSaveQuery,
      listSaved: mockListSaved,
      deleteSavedQuery: mockDeleteSavedQuery,
      listFolders: mockListFolders,
      createFolder: mockCreateFolder,
      deleteFolder: mockDeleteFolder,
      listRunConfigs: mockListRunConfigs,
      saveRunConfig: mockSaveRunConfig,
      deleteRunConfig: mockDeleteRunConfig,
      renameSavedQuery: mockRenameSavedQuery,
      duplicateSavedQuery: mockDuplicateSavedQuery,
      renameSaved: mockRenameSavedQuery,
    })),
  },
}));

vi.mock("../runtime/query-runtime", () => ({
  cancelQuery: (args: unknown) => mockRuntimeCancelQuery(args),
  explainQuery: (args: unknown) => mockRuntimeExplainQuery(args),
  executeQuery: vi.fn(),
  executeQueryMulti: vi.fn(),
  registerCacheInvalidation: vi.fn(() => vi.fn()),
}));

vi.mock("@/commons/stores/query-history.store", () => ({
  useQueryHistoryStore: {
    getState: () => ({
      addEntry: vi.fn(),
    }),
  },
}));

vi.mock("@/commons/stores/workspace.store", () => ({
  useWorkspaceStore: {
    getState: () => ({
      tabs: [],
    }),
  },
}));

vi.mock("../controllers/query-workspace.controller", () => ({
  setTabError: vi.fn(),
  setTabExplainPlan: vi.fn(),
  setTabExecutionStartedAt: vi.fn(),
  setTabMultiResults: vi.fn(),
  setTabResult: vi.fn(),
  setTabStatus: vi.fn(),
  setTabTiming: vi.fn(),
}));

import {
  useCancelQuery,
  useExplainPlan,
  useQueryHistory,
  useListSavedQueries,
  useListFolders,
  useListRunConfigs,
  useRenameSavedQuery,
} from "../queries/query.queries";

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });

  return function Wrapper({ children }: { children: ReactNode }) {
    return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
  };
}

describe("query.queries", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("useCancelQuery", () => {
    it("cancels query", async () => {
      mockRuntimeCancelQuery.mockResolvedValue(undefined);

      const { result } = renderHook(() => useCancelQuery(), {
        wrapper: createWrapper(),
      });

      result.current.mutate({ tabId: "tab-1", executionId: "exec-1" });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(mockRuntimeCancelQuery).toHaveBeenCalledWith({
        tabId: "tab-1",
        executionId: "exec-1",
      });
    });
  });

  describe("useExplainPlan", () => {
    it("gets explain plan", async () => {
      const mockPlan = { plan: "Seq Scan on users" };
      mockRuntimeExplainQuery.mockResolvedValue(mockPlan);

      const { result } = renderHook(() => useExplainPlan(), {
        wrapper: createWrapper(),
      });

      result.current.mutate({ connectionId: "conn-1", sql: "SELECT * FROM users", tabId: "tab-1" });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(mockRuntimeExplainQuery).toHaveBeenCalledWith({
        connectionId: "conn-1",
        sql: "SELECT * FROM users",
        tabId: "tab-1",
      });
    });
  });

  describe("useQueryHistory", () => {
    it("fetches query history", async () => {
      const mockHistory = [{ id: "1", sql: "SELECT 1" }];
      mockGetHistory.mockResolvedValue(mockHistory);

      const { result } = renderHook(() => useQueryHistory("conn-1"), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(mockGetHistory).toHaveBeenCalledWith("conn-1");
    });
  });

  describe("useListSavedQueries", () => {
    it("fetches saved queries", async () => {
      const mockQueries = [{ id: "1", name: "Query 1" }];
      mockListSaved.mockResolvedValue(mockQueries);

      const { result } = renderHook(() => useListSavedQueries("conn-1"), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(mockListSaved).toHaveBeenCalledWith("conn-1");
    });
  });

  describe("useListFolders", () => {
    it("fetches folders", async () => {
      const mockFolders = [{ id: "1", name: "Folder 1" }];
      mockListFolders.mockResolvedValue(mockFolders);

      const { result } = renderHook(() => useListFolders("conn-1"), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(mockListFolders).toHaveBeenCalledWith("conn-1");
    });
  });

  describe("useListRunConfigs", () => {
    it("fetches run configs", async () => {
      const mockConfigs = [{ id: "1", name: "Config 1" }];
      mockListRunConfigs.mockResolvedValue(mockConfigs);

      const { result } = renderHook(() => useListRunConfigs("conn-1"), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(mockListRunConfigs).toHaveBeenCalledWith("conn-1");
    });
  });

  describe("useRenameSavedQuery", () => {
    it("calls renameSaved directly and never calls deleteSavedQuery", async () => {
      mockRenameSavedQuery.mockResolvedValue(undefined);

      const { result } = renderHook(() => useRenameSavedQuery(), {
        wrapper: createWrapper(),
      });

      result.current.mutate({
        id: "sq-1",
        connectionId: "conn-1",
        newName: "New Query Name",
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(mockRenameSavedQuery).toHaveBeenCalledWith("sq-1", "New Query Name");
      expect(mockDeleteSavedQuery).not.toHaveBeenCalled();
    });
  });
});
