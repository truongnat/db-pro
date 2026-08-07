import { describe, expect, it, vi } from "vitest";

const { mockApiInvoke } = vi.hoisted(() => ({ mockApiInvoke: vi.fn() }));

vi.mock("@/commons/utils/api", () => ({
  apiInvoke: mockApiInvoke,
}));

import {
  UserManagementService,
  createUserManagementService,
} from "../services/user-management.service";

describe("UserManagementService", () => {
  describe("listUsers", () => {
    it("calls apiInvoke with connectionId", async () => {
      const users = [{ name: "alice", isSuper: true, canCreateDb: false, canCreateRole: false, canLogin: true }];
      mockApiInvoke.mockResolvedValueOnce(users);

      const svc = new UserManagementService();
      const result = await svc.listUsers("conn-1");

      expect(result).toEqual(users);
      expect(mockApiInvoke).toHaveBeenCalledWith("list_users", {
        req: { connectionId: "conn-1" },
      });
    });
  });

  describe("createRole", () => {
    it("calls apiInvoke with name and login flag", async () => {
      mockApiInvoke.mockResolvedValueOnce(undefined);

      const svc = new UserManagementService();
      await svc.createRole("conn-1", "bob", true);

      expect(mockApiInvoke).toHaveBeenCalledWith("create_role", {
        req: { connectionId: "conn-1", name: "bob", login: true },
      });
    });
  });

  describe("dropRole", () => {
    it("calls apiInvoke with role name", async () => {
      mockApiInvoke.mockResolvedValueOnce(undefined);

      const svc = new UserManagementService();
      await svc.dropRole("conn-1", "bob");

      expect(mockApiInvoke).toHaveBeenCalledWith("drop_role", {
        req: { connectionId: "conn-1", name: "bob" },
      });
    });
  });

  describe("listPrivileges", () => {
    it("calls apiInvoke with connectionId and roleName", async () => {
      const privs = [{ schema: "public", table: "users", privilegeType: "SELECT" }];
      mockApiInvoke.mockResolvedValueOnce(privs);

      const svc = new UserManagementService();
      const result = await svc.listPrivileges("conn-1", "bob");

      expect(result).toEqual(privs);
      expect(mockApiInvoke).toHaveBeenCalledWith("list_privileges", {
        req: { connectionId: "conn-1", roleName: "bob" },
      });
    });
  });

  describe("grantPrivilege", () => {
    it("calls apiInvoke with all params", async () => {
      mockApiInvoke.mockResolvedValueOnce(undefined);

      const svc = new UserManagementService();
      await svc.grantPrivilege("conn-1", "bob", "public", "users", "SELECT");

      expect(mockApiInvoke).toHaveBeenCalledWith("grant_privilege", {
        req: { connectionId: "conn-1", roleName: "bob", schema: "public", table: "users", privilege: "SELECT" },
      });
    });
  });

  describe("revokePrivilege", () => {
    it("calls apiInvoke with all params", async () => {
      mockApiInvoke.mockResolvedValueOnce(undefined);

      const svc = new UserManagementService();
      await svc.revokePrivilege("conn-1", "bob", "public", "users", "DELETE");

      expect(mockApiInvoke).toHaveBeenCalledWith("revoke_privilege", {
        req: { connectionId: "conn-1", roleName: "bob", schema: "public", table: "users", privilege: "DELETE" },
      });
    });
  });

  describe("createUserManagementService", () => {
    it("returns a UserManagementService instance", () => {
      const svc = createUserManagementService();
      expect(svc).toBeInstanceOf(UserManagementService);
    });
  });
});
