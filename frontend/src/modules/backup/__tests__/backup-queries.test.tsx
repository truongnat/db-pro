import { describe, expect, it, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactNode } from "react";

const { mockBackup, mockRestore } = vi.hoisted(() => ({
  mockBackup: vi.fn(),
  mockRestore: vi.fn(),
}));

vi.mock("@/app/app.module", () => ({
  container: {
    resolve: vi.fn(() => ({
      backup: mockBackup,
      restore: mockRestore,
    })),
  },
}));

import { useBackupDatabase, useRestoreDatabase } from "../queries/backup.queries";

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });

  return function Wrapper({ children }: { children: ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>
        {children}
      </QueryClientProvider>
    );
  };
}

describe("backup.queries", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("useBackupDatabase", () => {
    it("calls backup service with options", async () => {
      const mockResult = { success: true, path: "/backup/db.sql" };
      mockBackup.mockResolvedValue(mockResult);

      const { result } = renderHook(() => useBackupDatabase(), {
        wrapper: createWrapper(),
      });

      const options = { connectionId: "conn-1", path: "/backup/db.sql" };
      result.current.mutate(options);

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(mockBackup).toHaveBeenCalledWith(options);
      expect(result.current.data).toEqual(mockResult);
    });
  });

  describe("useRestoreDatabase", () => {
    it("calls restore service with options", async () => {
      const mockResult = { success: true };
      mockRestore.mockResolvedValue(mockResult);

      const { result } = renderHook(() => useRestoreDatabase(), {
        wrapper: createWrapper(),
      });

      const options = { connectionId: "conn-1", path: "/backup/db.sql" };
      result.current.mutate(options);

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(mockRestore).toHaveBeenCalledWith(options);
      expect(result.current.data).toEqual(mockResult);
    });
  });
});
