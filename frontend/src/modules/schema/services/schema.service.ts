import { apiInvoke } from "@/commons/utils/api";
import type { IntrospectResult, TableInfo } from "../types/schema.types";

export class SchemaService {
  async introspect(
    connectionId: string,
    forceRefresh?: boolean,
  ): Promise<IntrospectResult> {
    return apiInvoke<IntrospectResult>("introspect", {
      connection_id: connectionId,
      force_refresh: forceRefresh,
    });
  }

  async getTableInfo(
    connectionId: string,
    schema: string,
    table: string,
  ): Promise<TableInfo> {
    return apiInvoke<TableInfo>("get_table_info", {
      connection_id: connectionId,
      schema,
      table,
    });
  }

  async getTableDdl(
    connectionId: string,
    schema: string,
    table: string,
  ): Promise<string> {
    return apiInvoke<string>("get_table_ddl", {
      connection_id: connectionId,
      schema,
      table,
    });
  }

  async invalidateCache(connectionId: string): Promise<void> {
    return apiInvoke<void>("invalidate_cache", {
      connection_id: connectionId,
    });
  }
}

export function createSchemaService(): SchemaService {
  return new SchemaService();
}
