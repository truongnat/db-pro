import { apiInvoke } from "@/commons/utils/api";

import type { DatabaseUser, Privilege } from "../types/user.types";

export class UserManagementService {
  async listUsers(connectionId: string): Promise<DatabaseUser[]> {
    return apiInvoke<DatabaseUser[]>("list_users", {
      connection_id: connectionId,
    });
  }

  async createRole(
    connectionId: string,
    name: string,
    login: boolean,
  ): Promise<void> {
    return apiInvoke<void>("create_role", {
      connection_id: connectionId,
      name,
      login,
    });
  }

  async dropRole(connectionId: string, name: string): Promise<void> {
    return apiInvoke<void>("drop_role", {
      connection_id: connectionId,
      name,
    });
  }

  async listPrivileges(
    connectionId: string,
    roleName: string,
  ): Promise<Privilege[]> {
    return apiInvoke<Privilege[]>("list_privileges", {
      connection_id: connectionId,
      role_name: roleName,
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
      connection_id: connectionId,
      role_name: roleName,
      schema,
      table,
      privilege,
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
      connection_id: connectionId,
      role_name: roleName,
      schema,
      table,
      privilege,
    });
  }
}

export function createUserManagementService(): UserManagementService {
  return new UserManagementService();
}
