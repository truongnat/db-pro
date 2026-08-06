import { apiInvoke } from "@/commons/utils/api";
import type { DataDiff, IntrospectResult, ObjectDependency, PartitionInfo, SchemaDiff, TableInfo, TablespaceInfo } from "../types/schema.types";

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

  async executeDdl(
    connectionId: string,
    sql: string,
  ): Promise<{ affectedRows: number }> {
    return apiInvoke<{ affectedRows: number }>("execute_ddl", {
      connection_id: connectionId,
      sql,
    });
  }

  async invalidateCache(connectionId: string): Promise<void> {
    return apiInvoke<void>("invalidate_cache", {
      connection_id: connectionId,
    });
  }

  async diffSchemas(
    sourceId: string,
    targetId: string,
  ): Promise<SchemaDiff> {
    return apiInvoke<SchemaDiff>("diff_schemas", {
      source_id: sourceId,
      target_id: targetId,
    });
  }

  async diffTableData(
    sourceId: string,
    targetId: string,
    schema: string,
    table: string,
  ): Promise<DataDiff> {
    return apiInvoke<DataDiff>("diff_table_data", {
      source_id: sourceId,
      target_id: targetId,
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
      connection_id: connectionId,
      schema,
      object_name: objectName,
    });
  }

  async listPartitions(connectionId: string): Promise<PartitionInfo[]> {
    return apiInvoke<PartitionInfo[]>("list_partitions", {
      connection_id: connectionId,
    });
  }

  async listTablespaces(connectionId: string): Promise<TablespaceInfo[]> {
    return apiInvoke<TablespaceInfo[]>("list_tablespaces", {
      connection_id: connectionId,
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
      connection_id: connectionId,
      object_type: objectType,
      schema,
      old_name: oldName,
      new_name: newName,
    });
  }
}

export function createSchemaService(): SchemaService {
  return new SchemaService();
}
