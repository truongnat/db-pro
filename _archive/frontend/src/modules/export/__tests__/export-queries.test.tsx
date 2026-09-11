import { describe, expect, it, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactNode } from "react";

const { mockExportCsv, mockExportJson, mockExportExcel } = vi.hoisted(() => ({
  mockExportCsv: vi.fn(),
  mockExportJson: vi.fn(),
  mockExportExcel: vi.fn(),
}));

vi.mock("@/app/app.module", () => ({
  container: {
    resolve: vi.fn(() => ({
      exportCsv: mockExportCsv,
      exportJson: mockExportJson,
      exportExcel: mockExportExcel,
    })),
  },
}));

import { useExport } from "../queries/export.queries";

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

describe("export.queries", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("useExport", () => {
    it("calls exportCsv when format is csv", async () => {
      const mockResult = { success: true, rowCount: 10 };
      mockExportCsv.mockResolvedValue(mockResult);

      const { result } = renderHook(() => useExport("conn-1", "csv", "SELECT * FROM users"), {
        wrapper: createWrapper(),
      });

      result.current.mutate();

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(mockExportCsv).toHaveBeenCalledWith("conn-1", "SELECT * FROM users");
      expect(result.current.data).toEqual(mockResult);
    });

    it("calls exportJson when format is json", async () => {
      const mockResult = { success: true, rowCount: 5 };
      mockExportJson.mockResolvedValue(mockResult);

      const { result } = renderHook(() => useExport("conn-1", "json", "SELECT * FROM products"), {
        wrapper: createWrapper(),
      });

      result.current.mutate();

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(mockExportJson).toHaveBeenCalledWith("conn-1", "SELECT * FROM products");
    });

    it("calls exportExcel when format is excel", async () => {
      const mockResult = { success: true, rowCount: 20 };
      mockExportExcel.mockResolvedValue(mockResult);

      const { result } = renderHook(() => useExport("conn-1", "excel", "SELECT * FROM orders"), {
        wrapper: createWrapper(),
      });

      result.current.mutate();

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(mockExportExcel).toHaveBeenCalledWith("conn-1", "SELECT * FROM orders");
    });

    it("handles export errors gracefully", async () => {
      const mockError = new Error("Export failed");
      mockExportCsv.mockRejectedValue(mockError);

      const consoleSpy = vi.spyOn(console, "error").mockImplementation(() => {});

      const { result } = renderHook(() => useExport("conn-1", "csv", "SELECT * FROM users"), {
        wrapper: createWrapper(),
      });

      result.current.mutate();

      await waitFor(() => {
        expect(result.current.isError).toBe(true);
      });

      expect(result.current.error).toBe(mockError);
      expect(consoleSpy).toHaveBeenCalled();

      consoleSpy.mockRestore();
    });
  });
});
