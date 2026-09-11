import { describe, expect, it, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactNode } from "react";

const {
  mockIntrospect,
  mockGetTableInfo,
  mockGetTableDdl,
  mockInvalidateCache,
  mockExecuteDdl,
  mockDiffSchemas,
  mockDiffTableData,
  mockGetObjectDependencies,
  mockListPartitions,
  mockListTablespaces,
  mockRenameSchemaObject,
} = vi.hoisted(() => ({
  mockIntrospect: vi.fn(),
  mockGetTableInfo: vi.fn(),
  mockGetTableDdl: vi.fn(),
  mockInvalidateCache: vi.fn(),
  mockExecuteDdl: vi.fn(),
  mockDiffSchemas: vi.fn(),
  mockDiffTableData: vi.fn(),
  mockGetObjectDependencies: vi.fn(),
  mockListPartitions: vi.fn(),
  mockListTablespaces: vi.fn(),
  mockRenameSchemaObject: vi.fn(),
}));

vi.mock("@/app/app.module", () => ({
  container: {
    resolve: vi.fn(() => ({
      introspect: mockIntrospect,
      getTableInfo: mockGetTableInfo,
      getTableDdl: mockGetTableDdl,
      invalidateCache: mockInvalidateCache,
      executeDdl: mockExecuteDdl,
      diffSchemas: mockDiffSchemas,
      diffTableData: mockDiffTableData,
      getObjectDependencies: mockGetObjectDependencies,
      listPartitions: mockListPartitions,
      listTablespaces: mockListTablespaces,
      renameSchemaObject: mockRenameSchemaObject,
    })),
  },
}));

vi.mock("@/modules/query/stores/schema-catalog.store", () => ({
  useSchemaCatalogStore: {
    getState: () => ({
      invalidateConnection: vi.fn(),
    }),
  },
}));

import {
  useIntrospect,
  useTableInfo,
  useTableDdl,
  useInvalidateSchemaCache,
  useExecuteDdl,
  useDiffSchemas,
  useDiffTableData,
  useObjectDependencies,
  useListPartitions,
  useListTablespaces,
  useRenameSchemaObject,
} from "../queries/schema.queries";

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

describe("schema.queries", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("useIntrospect", () => {
    it("fetches schema when connectionId is provided", async () => {
      const mockResult = { tables: [], views: [] };
      mockIntrospect.mockResolvedValue(mockResult);

      const { result } = renderHook(() => useIntrospect("conn-1"), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(mockIntrospect).toHaveBeenCalledWith("conn-1");
    });

    it("does not fetch when connectionId is null", () => {
      const { result } = renderHook(() => useIntrospect(null), {
        wrapper: createWrapper(),
      });

      expect(result.current.fetchStatus).toBe("idle");
    });
  });

  describe("useTableInfo", () => {
    it("fetches table info when all params are provided", async () => {
      const mockResult = { columns: [], indexes: [] };
      mockGetTableInfo.mockResolvedValue(mockResult);

      const { result } = renderHook(() => useTableInfo("conn-1", "public", "users"), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(mockGetTableInfo).toHaveBeenCalledWith("conn-1", "public", "users");
    });

    it("does not fetch when any param is null", () => {
      const { result } = renderHook(() => useTableInfo("conn-1", null, "users"), {
        wrapper: createWrapper(),
      });

      expect(result.current.fetchStatus).toBe("idle");
    });
  });

  describe("useTableDdl", () => {
    it("fetches DDL when enabled and all params provided", async () => {
      const mockResult = "CREATE TABLE users (...)";
      mockGetTableDdl.mockResolvedValue(mockResult);

      const { result } = renderHook(() => useTableDdl("conn-1", "public", "users", true), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(mockGetTableDdl).toHaveBeenCalledWith("conn-1", "public", "users");
    });

    it("does not fetch when disabled", () => {
      const { result } = renderHook(() => useTableDdl("conn-1", "public", "users", false), {
        wrapper: createWrapper(),
      });

      expect(result.current.fetchStatus).toBe("idle");
    });
  });

  describe("useInvalidateSchemaCache", () => {
    it("invalidates cache", async () => {
      mockInvalidateCache.mockResolvedValue(undefined);

      const { result } = renderHook(() => useInvalidateSchemaCache("conn-1"), {
        wrapper: createWrapper(),
      });

      result.current.mutate();

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(mockInvalidateCache).toHaveBeenCalledWith("conn-1");
    });
  });

  describe("useExecuteDdl", () => {
    it("executes DDL and invalidates cache", async () => {
      const mockResult = { affectedRows: 0 };
      mockExecuteDdl.mockResolvedValue(mockResult);

      const { result } = renderHook(() => useExecuteDdl("conn-1"), {
        wrapper: createWrapper(),
      });

      result.current.mutate("ALTER TABLE users ADD COLUMN email TEXT");

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(mockExecuteDdl).toHaveBeenCalledWith(
        "conn-1",
        "ALTER TABLE users ADD COLUMN email TEXT",
      );
    });
  });

  describe("useDiffSchemas", () => {
    it("fetches schema diff when enabled", async () => {
      const mockResult = { added: [], removed: [], modified: [] };
      mockDiffSchemas.mockResolvedValue(mockResult);

      const { result } = renderHook(() => useDiffSchemas("conn-1", "conn-2", true), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(mockDiffSchemas).toHaveBeenCalledWith("conn-1", "conn-2");
    });

    it("does not fetch when disabled", () => {
      const { result } = renderHook(() => useDiffSchemas("conn-1", "conn-2", false), {
        wrapper: createWrapper(),
      });

      expect(result.current.fetchStatus).toBe("idle");
    });
  });

  describe("useDiffTableData", () => {
    it("fetches data diff when enabled and all params provided", async () => {
      const mockResult = { added: [], removed: [], modified: [] };
      mockDiffTableData.mockResolvedValue(mockResult);

      const { result } = renderHook(
        () => useDiffTableData("conn-1", "conn-2", "public", "users", true),
        { wrapper: createWrapper() },
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(mockDiffTableData).toHaveBeenCalledWith("conn-1", "conn-2", "public", "users");
    });

    it("does not fetch when any param is null", () => {
      const { result } = renderHook(
        () => useDiffTableData("conn-1", "conn-2", null, "users", true),
        { wrapper: createWrapper() },
      );

      expect(result.current.fetchStatus).toBe("idle");
    });
  });

  describe("useObjectDependencies", () => {
    it("fetches dependencies when enabled and all params provided", async () => {
      const mockResult = [{ dependentSchema: "public", dependentName: "view1" }];
      mockGetObjectDependencies.mockResolvedValue(mockResult);

      const { result } = renderHook(
        () => useObjectDependencies("conn-1", "public", "users", true),
        { wrapper: createWrapper() },
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(mockGetObjectDependencies).toHaveBeenCalledWith("conn-1", "public", "users");
    });
  });

  describe("useListPartitions", () => {
    it("fetches partitions when enabled", async () => {
      const mockResult = [{ name: "part1", parent: "users" }];
      mockListPartitions.mockResolvedValue(mockResult);

      const { result } = renderHook(() => useListPartitions("conn-1", true), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(mockListPartitions).toHaveBeenCalledWith("conn-1");
    });
  });

  describe("useListTablespaces", () => {
    it("fetches tablespaces when enabled", async () => {
      const mockResult = [{ name: "pg_default", location: "/data" }];
      mockListTablespaces.mockResolvedValue(mockResult);

      const { result } = renderHook(() => useListTablespaces("conn-1", true), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(mockListTablespaces).toHaveBeenCalledWith("conn-1");
    });
  });

  describe("useRenameSchemaObject", () => {
    it("renames object and invalidates cache", async () => {
      mockRenameSchemaObject.mockResolvedValue(undefined);

      const { result } = renderHook(() => useRenameSchemaObject("conn-1"), {
        wrapper: createWrapper(),
      });

      result.current.mutate({
        objectType: "TABLE",
        schema: "public",
        oldName: "users",
        newName: "accounts",
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(mockRenameSchemaObject).toHaveBeenCalledWith(
        "conn-1",
        "TABLE",
        "public",
        "users",
        "accounts",
      );
    });
  });
});
