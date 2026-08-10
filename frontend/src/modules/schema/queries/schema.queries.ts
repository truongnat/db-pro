import { useMutation, useQuery, useQueryClient, type QueryClient } from "@tanstack/react-query";

import { container } from "@/app/app.module";
import { SERVICE_NAMES, type ISchemaService } from "@/commons/di/registry";
import { useSchemaCatalogStore } from "@/modules/query/stores/schema-catalog.store";

import type {
  DataDiff,
  IntrospectResult,
  ObjectDependency,
  PartitionInfo,
  SchemaDiff,
  TableInfo,
  TablespaceInfo,
} from "../types/schema.types";

const QUERY_KEYS = {
  introspect: (connectionId: string) => ["schema-introspect", connectionId] as const,
  tableInfo: (connectionId: string, schema: string, table: string) =>
    ["schema-table-info", connectionId, schema, table] as const,
  tableDdl: (connectionId: string, schema: string, table: string) =>
    ["schema-table-ddl", connectionId, schema, table] as const,
  schemaDiff: (sourceId: string, targetId: string) => ["schema-diff", sourceId, targetId] as const,
  dataDiff: (sourceId: string, targetId: string, schema: string, table: string) =>
    ["data-diff", sourceId, targetId, schema, table] as const,
  dependencies: (connectionId: string, schema: string, objectName: string) =>
    ["object-dependencies", connectionId, schema, objectName] as const,
  partitions: (connectionId: string) => ["partitions", connectionId] as const,
  tablespaces: (connectionId: string) => ["tablespaces", connectionId] as const,
};

function getSchemaService() {
  return container.resolve<ISchemaService>(SERVICE_NAMES.SCHEMA_SERVICE);
}

/** Imperatively force-refresh introspection for a specific connection.
 *  Invalidates backend cache, schema catalog, and React Query cache. */
export async function refreshIntrospection(queryClient: QueryClient, connectionId: string) {
  // 1. Invalidate backend introspection cache so next call bypasses it
  await getSchemaService().invalidateCache(connectionId);
  // 2. Invalidate schema catalog store (has its own client-side cache)
  useSchemaCatalogStore.getState().invalidateConnection(connectionId);
  // 3. Invalidate React Query cache — triggers refetch with fresh data
  queryClient.invalidateQueries({ queryKey: QUERY_KEYS.introspect(connectionId) });
}

export function useIntrospect(connectionId: string | null) {
  return useQuery({
    queryKey: QUERY_KEYS.introspect(connectionId ?? ""),
    queryFn: () => getSchemaService().introspect(connectionId!) as Promise<IntrospectResult>,
    enabled: !!connectionId,
    staleTime: 5 * 60 * 1000,
  });
}

export function useTableInfo(
  connectionId: string | null,
  schema: string | null,
  table: string | null,
) {
  return useQuery({
    queryKey: QUERY_KEYS.tableInfo(connectionId ?? "", schema ?? "", table ?? ""),
    queryFn: () =>
      getSchemaService().getTableInfo(connectionId!, schema!, table!) as Promise<TableInfo>,
    enabled: !!connectionId && !!schema && !!table,
  });
}

export function useTableDdl(
  connectionId: string | null,
  schema: string | null,
  table: string | null,
  enabled: boolean,
) {
  return useQuery({
    queryKey: QUERY_KEYS.tableDdl(connectionId ?? "", schema ?? "", table ?? ""),
    queryFn: () => getSchemaService().getTableDdl(connectionId!, schema!, table!),
    enabled: enabled && !!connectionId && !!schema && !!table,
  });
}

export function useInvalidateSchemaCache(connectionId: string | null) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: () => getSchemaService().invalidateCache(connectionId!),
    onSuccess: () => {
      if (connectionId) {
        qc.invalidateQueries({ queryKey: QUERY_KEYS.introspect(connectionId) });
        useSchemaCatalogStore.getState().invalidateConnection(connectionId);
      }
    },
    onError: (err: unknown) => {
      console.error("[Schema] Cache invalidation failed:", err);
    },
  });
}

function invalidateAllSchemaCaches(qc: ReturnType<typeof useQueryClient>, connectionId: string) {
  qc.invalidateQueries({ queryKey: QUERY_KEYS.introspect(connectionId) });
  qc.invalidateQueries({ queryKey: ["schema-table-info"] });
  qc.invalidateQueries({ queryKey: ["schema-table-ddl"] });
  qc.invalidateQueries({ queryKey: ["object-dependencies"] });
  useSchemaCatalogStore.getState().invalidateConnection(connectionId);
}

export function useExecuteDdl(connectionId: string | null) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (sql: string) =>
      getSchemaService().executeDdl(connectionId!, sql) as Promise<{
        affectedRows: number;
      }>,
    onSuccess: () => {
      if (connectionId) {
        invalidateAllSchemaCaches(qc, connectionId);
      }
    },
  });
}

export function useExecuteDdlBatch(connectionId: string | null) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (statements: string[]) =>
      getSchemaService().executeDdlBatch(connectionId!, statements) as Promise<{
        affectedRows: number;
      }>,
    onSuccess: () => {
      if (connectionId) {
        invalidateAllSchemaCaches(qc, connectionId);
      }
    },
  });
}

export function useDiffSchemas(sourceId: string | null, targetId: string | null, enabled: boolean) {
  return useQuery({
    queryKey: QUERY_KEYS.schemaDiff(sourceId ?? "", targetId ?? ""),
    queryFn: () => getSchemaService().diffSchemas(sourceId!, targetId!) as Promise<SchemaDiff>,
    enabled: enabled && !!sourceId && !!targetId,
  });
}

export function useDiffTableData(
  sourceId: string | null,
  targetId: string | null,
  schema: string | null,
  table: string | null,
  enabled: boolean,
) {
  return useQuery({
    queryKey: QUERY_KEYS.dataDiff(sourceId ?? "", targetId ?? "", schema ?? "", table ?? ""),
    queryFn: () =>
      getSchemaService().diffTableData(sourceId!, targetId!, schema!, table!) as Promise<DataDiff>,
    enabled: enabled && !!sourceId && !!targetId && !!schema && !!table,
  });
}

export function useObjectDependencies(
  connectionId: string | null,
  schema: string | null,
  objectName: string | null,
  enabled: boolean,
) {
  return useQuery({
    queryKey: QUERY_KEYS.dependencies(connectionId ?? "", schema ?? "", objectName ?? ""),
    queryFn: () =>
      getSchemaService().getObjectDependencies(connectionId!, schema!, objectName!) as Promise<
        ObjectDependency[]
      >,
    enabled: enabled && !!connectionId && !!schema && !!objectName,
  });
}

export function useListPartitions(connectionId: string | null, enabled: boolean) {
  return useQuery({
    queryKey: QUERY_KEYS.partitions(connectionId ?? ""),
    queryFn: () => getSchemaService().listPartitions(connectionId!) as Promise<PartitionInfo[]>,
    enabled: enabled && !!connectionId,
  });
}

export function useListTablespaces(connectionId: string | null, enabled: boolean) {
  return useQuery({
    queryKey: QUERY_KEYS.tablespaces(connectionId ?? ""),
    queryFn: () => getSchemaService().listTablespaces(connectionId!) as Promise<TablespaceInfo[]>,
    enabled: enabled && !!connectionId,
  });
}

export function useRenameSchemaObject(connectionId: string | null) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({
      objectType,
      schema,
      oldName,
      newName,
    }: {
      objectType: string;
      schema: string;
      oldName: string;
      newName: string;
    }) =>
      getSchemaService().renameSchemaObject(connectionId!, objectType, schema, oldName, newName),
    onSuccess: () => {
      if (connectionId) {
        qc.invalidateQueries({ queryKey: QUERY_KEYS.introspect(connectionId) });
        useSchemaCatalogStore.getState().invalidateConnection(connectionId);
      }
    },
  });
}
