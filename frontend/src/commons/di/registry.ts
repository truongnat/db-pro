export const SERVICE_NAMES = {
  CONNECTION_SERVICE: "CONNECTION_SERVICE",
  QUERY_SERVICE: "QUERY_SERVICE",
  SCHEMA_SERVICE: "SCHEMA_SERVICE",
  DATA_GRID_SERVICE: "DATA_GRID_SERVICE",
  EXPORT_SERVICE: "EXPORT_SERVICE",
  USER_MANAGEMENT_SERVICE: "USER_MANAGEMENT_SERVICE",
  BACKUP_SERVICE: "BACKUP_SERVICE",
} as const;

export type ServiceName = (typeof SERVICE_NAMES)[keyof typeof SERVICE_NAMES];

export interface IConnectionService {
  list(): Promise<unknown[]>;
  get(id: string): Promise<unknown | null>;
  create(config: unknown, password: string): Promise<unknown>;
  update(id: string, config: unknown, password?: string): Promise<void>;
  delete(id: string): Promise<void>;
  test(config: unknown, password: string): Promise<void>;
  connect(id: string): Promise<void>;
  disconnect(id: string): Promise<void>;
  testSshTunnel(config: unknown): Promise<void>;
}

export interface IQueryService {
  execute(connectionId: string, sql: string): Promise<unknown>;
  cancel(connectionId: string): Promise<void>;
  executeMulti(connectionId: string, sql: string): Promise<unknown>;
  explain(connectionId: string, sql: string): Promise<unknown>;
  getHistory(connectionId: string, limit?: number): Promise<unknown[]>;
  save(
    connectionId: string,
    name: string,
    sql: string,
    folder?: string,
  ): Promise<unknown>;
  listSaved(connectionId: string): Promise<unknown[]>;
  deleteSaved(id: string): Promise<void>;
  createFolder(connectionId: string, name: string): Promise<unknown>;
  listFolders(connectionId: string): Promise<unknown[]>;
  deleteFolder(id: string): Promise<void>;
  saveRunConfig(
    connectionId: string,
    name: string,
    sql: string,
    timeoutMs: number,
    maxRows: number,
  ): Promise<unknown>;
  listRunConfigs(connectionId: string): Promise<unknown[]>;
  deleteRunConfig(id: string): Promise<void>;
}

export interface ISchemaService {
  introspect(connectionId: string, forceRefresh?: boolean): Promise<unknown>;
  getTableInfo(
    connectionId: string,
    schema: string,
    table: string,
  ): Promise<unknown>;
  getTableDdl(
    connectionId: string,
    schema: string,
    table: string,
  ): Promise<string>;
  executeDdl(connectionId: string, sql: string): Promise<unknown>;
  invalidateCache(connectionId: string): Promise<void>;
  diffSchemas(sourceId: string, targetId: string): Promise<unknown>;
  diffTableData(
    sourceId: string,
    targetId: string,
    schema: string,
    table: string,
  ): Promise<unknown>;
  getObjectDependencies(
    connectionId: string,
    schema: string,
    objectName: string,
  ): Promise<unknown>;
  listPartitions(connectionId: string): Promise<unknown>;
  listTablespaces(connectionId: string): Promise<unknown>;
  renameSchemaObject(
    connectionId: string,
    objectType: string,
    schema: string,
    oldName: string,
    newName: string,
  ): Promise<void>;
}

export interface IDataGridService {
  fetchRows(connectionId: string, request: unknown): Promise<unknown>;
  insertRow(connectionId: string, request: unknown): Promise<unknown>;
  updateRow(connectionId: string, request: unknown): Promise<unknown>;
  deleteRow(connectionId: string, request: unknown): Promise<unknown>;
}

export interface IExportService {
  exportCsv(connectionId: string, sql: string): Promise<unknown>;
  exportJson(connectionId: string, sql: string): Promise<unknown>;
  exportExcel(connectionId: string, sql: string): Promise<unknown>;
}

export interface IUserManagementService {
  listUsers(connectionId: string): Promise<unknown[]>;
  createRole(connectionId: string, name: string, login: boolean): Promise<void>;
  dropRole(connectionId: string, name: string): Promise<void>;
  listPrivileges(connectionId: string, roleName: string): Promise<unknown[]>;
  grantPrivilege(
    connectionId: string,
    roleName: string,
    schema: string,
    table: string,
    privilege: string,
  ): Promise<void>;
  revokePrivilege(
    connectionId: string,
    roleName: string,
    schema: string,
    table: string,
    privilege: string,
  ): Promise<void>;
}

export interface IBackupService {
  backup(options: unknown): Promise<unknown>;
  restore(options: unknown): Promise<void>;
}

export interface ServiceRegistry {
  [SERVICE_NAMES.CONNECTION_SERVICE]: IConnectionService;
  [SERVICE_NAMES.QUERY_SERVICE]: IQueryService;
  [SERVICE_NAMES.SCHEMA_SERVICE]: ISchemaService;
  [SERVICE_NAMES.DATA_GRID_SERVICE]: IDataGridService;
  [SERVICE_NAMES.EXPORT_SERVICE]: IExportService;
  [SERVICE_NAMES.USER_MANAGEMENT_SERVICE]: IUserManagementService;
  [SERVICE_NAMES.BACKUP_SERVICE]: IBackupService;
}
