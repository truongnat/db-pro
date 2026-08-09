import { describe, expect, it, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactNode } from "react";

const { mockFetchRows, mockInsertRow, mockUpdateRow, mockDeleteRow } = vi.hoisted(() => ({
  mockFetchRows: vi.fn(),
  mockInsertRow: vi.fn(),
  mockUpdateRow: vi.fn(),
  mockDeleteRow: vi.fn(),
}));

vi.mock("@/app/app.module", () => ({
  container: {
    resolve: vi.fn(() => ({
      fetchRows: mockFetchRows,
      insertRow: mockInsertRow,
      updateRow: mockUpdateRow,
      deleteRow: mockDeleteRow,
    })),
  },
}));

import {
  useTableRows,
  useInsertRow,
  useUpdateRow,
  useDeleteRow,
} from "../queries/data-grid.queries";

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

describe("data-grid.queries", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("useTableRows", () => {
    it("fetches rows when connectionId and request are provided", async () => {
      const mockResult = { rows: [], totalCount: 0, page: 1, pageSize: 50 };
      mockFetchRows.mockResolvedValue(mockResult);

      const { result } = renderHook(
        () =>
          useTableRows("conn-1", {
            table: "users",
            schema: "public",
            page: 1,
            pageSize: 50,
            filters: [],
            sorts: [],
          }),
        { wrapper: createWrapper() },
      );

      expect(result.current.isLoading).toBe(true);

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(mockFetchRows).toHaveBeenCalledWith("conn-1", {
        table: "users",
        schema: "public",
        page: 1,
        pageSize: 50,
        filters: [],
        sorts: [],
      });
    });

    it("does not fetch when connectionId is null", () => {
      const { result } = renderHook(
        () =>
          useTableRows(null, {
            table: "users",
            schema: "public",
            page: 1,
            pageSize: 50,
            filters: [],
            sorts: [],
          }),
        { wrapper: createWrapper() },
      );

      expect(result.current.fetchStatus).toBe("idle");
      expect(mockFetchRows).not.toHaveBeenCalled();
    });

    it("does not fetch when request is null", () => {
      const { result } = renderHook(() => useTableRows("conn-1", null), {
        wrapper: createWrapper(),
      });

      expect(result.current.fetchStatus).toBe("idle");
      expect(mockFetchRows).not.toHaveBeenCalled();
    });
  });

  describe("useInsertRow", () => {
    it("inserts row and invalidates cache on success", async () => {
      const mockResult = { success: true };
      mockInsertRow.mockResolvedValue(mockResult);

      const { result } = renderHook(
        () =>
          useInsertRow("conn-1", {
            table: "users",
            schema: "public",
            page: 1,
            pageSize: 50,
            filters: [],
            sorts: [],
          }),
        { wrapper: createWrapper() },
      );

      result.current.mutate({ table: "users", schema: "public", row: { id: 1, name: "Alice" } });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(mockInsertRow).toHaveBeenCalledWith("conn-1", {
        table: "users",
        schema: "public",
        row: { id: 1, name: "Alice" },
      });
    });
  });

  describe("useUpdateRow", () => {
    it("updates row and invalidates cache on success", async () => {
      const mockResult = { success: true };
      mockUpdateRow.mockResolvedValue(mockResult);

      const { result } = renderHook(
        () =>
          useUpdateRow("conn-1", {
            table: "users",
            schema: "public",
            page: 1,
            pageSize: 50,
            filters: [],
            sorts: [],
          }),
        { wrapper: createWrapper() },
      );

      result.current.mutate({ table: "users", schema: "public", row: { id: 1, name: "Bob" } });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(mockUpdateRow).toHaveBeenCalledWith("conn-1", {
        table: "users",
        schema: "public",
        row: { id: 1, name: "Bob" },
      });
    });
  });

  describe("useDeleteRow", () => {
    it("deletes row and invalidates cache on success", async () => {
      const mockResult = { success: true };
      mockDeleteRow.mockResolvedValue(mockResult);

      const { result } = renderHook(
        () =>
          useDeleteRow("conn-1", {
            table: "users",
            schema: "public",
            page: 1,
            pageSize: 50,
            filters: [],
            sorts: [],
          }),
        { wrapper: createWrapper() },
      );

      result.current.mutate({ table: "users", schema: "public", row: { id: 1 } });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(mockDeleteRow).toHaveBeenCalledWith("conn-1", {
        table: "users",
        schema: "public",
        row: { id: 1 },
      });
    });
  });
});
