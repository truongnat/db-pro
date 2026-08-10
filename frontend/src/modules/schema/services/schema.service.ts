import { apiInvoke } from "@/commons/utils/api";
import type {
  DataDiff,
  IntrospectResult,
  ObjectDependency,
  PartitionInfo,
  SchemaDiff,
  TableInfo,
  TablespaceInfo,
} from "../types/schema.types";

export class SchemaService {
  async introspect(connectionId: string, forceRefresh?: boolean): Promise<IntrospectResult> {
    return apiInvoke<IntrospectResult>("introspect", {
      connectionId,
      forceRefresh,
    });
  }

  async getTableInfo(connectionId: string, schema: string, table: string): Promise<TableInfo> {
    return apiInvoke<TableInfo>("get_table_info", {
      connectionId,
      schema,
      table,
    });
  }

  async getTableDdl(connectionId: string, schema: string, table: string): Promise<string> {
    return apiInvoke<string>("get_table_ddl", {
      connectionId,
      schema,
      table,
    });
  }

  async executeDdl(connectionId: string, sql: string): Promise<{ affectedRows: number }> {
    return apiInvoke<{ affectedRows: number }>("execute_ddl", {
      connectionId,
      sql,
    });
  }

  async executeDdlBatch(
    connectionId: string,
    statements: string[],
  ): Promise<{ affectedRows: number }> {
    return apiInvoke<{ affectedRows: number }>("execute_ddl_batch", {
      connectionId,
      statements,
    });
  }

  async invalidateCache(connectionId: string): Promise<void> {
    return apiInvoke<void>("invalidate_cache", {
      connectionId,
    });
  }

  async diffSchemas(sourceId: string, targetId: string): Promise<SchemaDiff> {
    return apiInvoke<SchemaDiff>("diff_schemas", {
      sourceId,
      targetId,
    });
  }

  async diffTableData(
    sourceId: string,
    targetId: string,
    schema: string,
    table: string,
  ): Promise<DataDiff> {
    return apiInvoke<DataDiff>("diff_table_data", {
      sourceId,
      targetId,
      schema,
      table,
    });
  }

  async getObjectDependencies(
    connectionId: string,
    schema: string,
    objectName: string,
  ): Promise<ObjectDependency[]> {
    return apiInvoke<ObjectDependency[]>("get_object_dependencies", {
      connectionId,
      schema,
      objectName,
    });
  }

  async listPartitions(connectionId: string): Promise<PartitionInfo[]> {
    return apiInvoke<PartitionInfo[]>("list_partitions", {
      connectionId,
    });
  }

  async listTablespaces(connectionId: string): Promise<TablespaceInfo[]> {
    return apiInvoke<TablespaceInfo[]>("list_tablespaces", {
      connectionId,
    });
  }

  async renameSchemaObject(
    connectionId: string,
    objectType: string,
    schema: string,
    oldName: string,
    newName: string,
  ): Promise<void> {
    return apiInvoke<void>("rename_schema_object", {
      connectionId,
      objectType,
      schema,
      oldName,
      newName,
    });
  }
}

export function createSchemaService(): SchemaService {
  return new SchemaService();
}
