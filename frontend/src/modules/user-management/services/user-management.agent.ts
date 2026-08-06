import type { DatabaseUser, Privilege } from "../types/user.types";

export class MockUserManagementService {
  private users: DatabaseUser[] = [
    { name: "postgres", isSuper: true, canCreateDb: true, canCreateRole: true, canLogin: true },
    { name: "app_user", isSuper: false, canCreateDb: false, canCreateRole: false, canLogin: true },
    { name: "readonly", isSuper: false, canCreateDb: false, canCreateRole: false, canLogin: true },
  ];

  private privileges: Map<string, Privilege[]> = new Map([
    ["app_user", [
      { schema: "public", table: "users", privilegeType: "SELECT" },
      { schema: "public", table: "users", privilegeType: "INSERT" },
      { schema: "public", table: "orders", privilegeType: "SELECT" },
    ]],
  ]);

  async listUsers(_connectionId: string): Promise<DatabaseUser[]> {
    return this.users;
  }

  async createRole(
    _connectionId: string,
    name: string,
    login: boolean,
  ): Promise<void> {
    this.users.push({
      name,
      isSuper: false,
      canCreateDb: false,
      canCreateRole: false,
      canLogin: login,
    });
  }

  async dropRole(_connectionId: string, name: string): Promise<void> {
    this.users = this.users.filter((u) => u.name !== name);
    this.privileges.delete(name);
  }

  async listPrivileges(
    _connectionId: string,
    roleName: string,
  ): Promise<Privilege[]> {
    return this.privileges.get(roleName) ?? [];
  }

  async grantPrivilege(
    _connectionId: string,
    roleName: string,
    schema: string,
    table: string,
    privilege: string,
  ): Promise<void> {
    const list = this.privileges.get(roleName) ?? [];
    list.push({ schema, table, privilegeType: privilege });
    this.privileges.set(roleName, list);
  }

  async revokePrivilege(
    _connectionId: string,
    roleName: string,
    schema: string,
    table: string,
    privilege: string,
  ): Promise<void> {
    const list = this.privileges.get(roleName) ?? [];
    this.privileges.set(
      roleName,
      list.filter(
        (p) =>
          !(p.schema === schema && p.table === table && p.privilegeType === privilege),
      ),
    );
  }
}
