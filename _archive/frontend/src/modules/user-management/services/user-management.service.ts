import { apiInvoke } from "@/commons/utils/api";

import type { DatabaseUser, Privilege } from "../types/user.types";

export class UserManagementService {
  async listUsers(connectionId: string): Promise<DatabaseUser[]> {
    return apiInvoke<DatabaseUser[]>("list_users", {
      req: { connectionId },
    });
  }

  async createRole(connectionId: string, name: string, login: boolean): Promise<void> {
    return apiInvoke<void>("create_role", {
      req: { connectionId, name, login },
    });
  }

  async dropRole(connectionId: string, name: string): Promise<void> {
    return apiInvoke<void>("drop_role", {
      req: { connectionId, name },
    });
  }

  async listPrivileges(connectionId: string, roleName: string): Promise<Privilege[]> {
    return apiInvoke<Privilege[]>("list_privileges", {
      req: { connectionId, roleName },
    });
  }

  async grantPrivilege(
    connectionId: string,
    roleName: string,
    schema: string,
    table: string,
    privilege: string,
  ): Promise<void> {
    return apiInvoke<void>("grant_privilege", {
      req: { connectionId, roleName, schema, table, privilege },
    });
  }

  async revokePrivilege(
    connectionId: string,
    roleName: string,
    schema: string,
    table: string,
    privilege: string,
  ): Promise<void> {
    return apiInvoke<void>("revoke_privilege", {
      req: { connectionId, roleName, schema, table, privilege },
    });
  }
}

export function createUserManagementService(): UserManagementService {
  return new UserManagementService();
}
