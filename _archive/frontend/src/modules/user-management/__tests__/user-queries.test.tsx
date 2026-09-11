import { describe, expect, it, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactNode } from "react";

const {
  mockListUsers,
  mockCreateRole,
  mockDropRole,
  mockListPrivileges,
  mockGrantPrivilege,
  mockRevokePrivilege,
} = vi.hoisted(() => ({
  mockListUsers: vi.fn(),
  mockCreateRole: vi.fn(),
  mockDropRole: vi.fn(),
  mockListPrivileges: vi.fn(),
  mockGrantPrivilege: vi.fn(),
  mockRevokePrivilege: vi.fn(),
}));

vi.mock("@/app/app.module", () => ({
  container: {
    resolve: vi.fn(() => ({
      listUsers: mockListUsers,
      createRole: mockCreateRole,
      dropRole: mockDropRole,
      listPrivileges: mockListPrivileges,
      grantPrivilege: mockGrantPrivilege,
      revokePrivilege: mockRevokePrivilege,
    })),
  },
}));

import {
  useListUsers,
  useCreateRole,
  useDropRole,
  useListPrivileges,
  useGrantPrivilege,
  useRevokePrivilege,
} from "../queries/user.queries";

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

describe("user.queries", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("useListUsers", () => {
    it("fetches users when connectionId is provided", async () => {
      const mockUsers = [{ name: "admin", login: true }];
      mockListUsers.mockResolvedValue(mockUsers);

      const { result } = renderHook(() => useListUsers("conn-1"), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(mockListUsers).toHaveBeenCalledWith("conn-1");
      expect(result.current.data).toEqual(mockUsers);
    });

    it("does not fetch when connectionId is null", () => {
      const { result } = renderHook(() => useListUsers(null), {
        wrapper: createWrapper(),
      });

      expect(result.current.fetchStatus).toBe("idle");
      expect(mockListUsers).not.toHaveBeenCalled();
    });
  });

  describe("useCreateRole", () => {
    it("creates role and invalidates cache", async () => {
      mockCreateRole.mockResolvedValue(undefined);

      const { result } = renderHook(() => useCreateRole("conn-1"), {
        wrapper: createWrapper(),
      });

      result.current.mutate({ name: "new_role", login: false });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(mockCreateRole).toHaveBeenCalledWith("conn-1", "new_role", false);
    });
  });

  describe("useDropRole", () => {
    it("drops role and invalidates cache", async () => {
      mockDropRole.mockResolvedValue(undefined);

      const { result } = renderHook(() => useDropRole("conn-1"), {
        wrapper: createWrapper(),
      });

      result.current.mutate("old_role");

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(mockDropRole).toHaveBeenCalledWith("conn-1", "old_role");
    });
  });

  describe("useListPrivileges", () => {
    it("fetches privileges when connectionId and roleName are provided", async () => {
      const mockPrivileges = [{ schema: "public", table: "users", privilege: "SELECT" }];
      mockListPrivileges.mockResolvedValue(mockPrivileges);

      const { result } = renderHook(() => useListPrivileges("conn-1", "admin"), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(mockListPrivileges).toHaveBeenCalledWith("conn-1", "admin");
      expect(result.current.data).toEqual(mockPrivileges);
    });

    it("does not fetch when roleName is null", () => {
      const { result } = renderHook(() => useListPrivileges("conn-1", null), {
        wrapper: createWrapper(),
      });

      expect(result.current.fetchStatus).toBe("idle");
      expect(mockListPrivileges).not.toHaveBeenCalled();
    });
  });

  describe("useGrantPrivilege", () => {
    it("grants privilege and invalidates cache", async () => {
      mockGrantPrivilege.mockResolvedValue(undefined);

      const { result } = renderHook(() => useGrantPrivilege("conn-1", "admin"), {
        wrapper: createWrapper(),
      });

      result.current.mutate({ schema: "public", table: "users", privilege: "SELECT" });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(mockGrantPrivilege).toHaveBeenCalledWith(
        "conn-1",
        "admin",
        "public",
        "users",
        "SELECT",
      );
    });
  });

  describe("useRevokePrivilege", () => {
    it("revokes privilege and invalidates cache", async () => {
      mockRevokePrivilege.mockResolvedValue(undefined);

      const { result } = renderHook(() => useRevokePrivilege("conn-1", "admin"), {
        wrapper: createWrapper(),
      });

      result.current.mutate({ schema: "public", table: "users", privilege: "SELECT" });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(mockRevokePrivilege).toHaveBeenCalledWith(
        "conn-1",
        "admin",
        "public",
        "users",
        "SELECT",
      );
    });
  });
});
