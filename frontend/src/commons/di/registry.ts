export const SERVICE_NAMES = {
  CONNECTION_SERVICE: "CONNECTION_SERVICE",
  QUERY_SERVICE: "QUERY_SERVICE",
  SCHEMA_SERVICE: "SCHEMA_SERVICE",
  DATA_GRID_SERVICE: "DATA_GRID_SERVICE",
  EXPORT_SERVICE: "EXPORT_SERVICE",
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
}

export interface IQueryService {
  execute(connectionId: string, sql: string): Promise<unknown>;
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
  invalidateCache(connectionId: string): Promise<void>;
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

export interface ServiceRegistry {
  [SERVICE_NAMES.CONNECTION_SERVICE]: IConnectionService;
  [SERVICE_NAMES.QUERY_SERVICE]: IQueryService;
  [SERVICE_NAMES.SCHEMA_SERVICE]: ISchemaService;
  [SERVICE_NAMES.DATA_GRID_SERVICE]: IDataGridService;
  [SERVICE_NAMES.EXPORT_SERVICE]: IExportService;
}
