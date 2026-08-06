import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { container } from "@/app/app.module";
import { SERVICE_NAMES, type ISchemaService } from "@/commons/di/registry";

import type { IntrospectResult, TableInfo } from "../types/schema.types";

const QUERY_KEYS = {
  introspect: (connectionId: string) =>
    ["schema-introspect", connectionId] as const,
  tableInfo: (connectionId: string, schema: string, table: string) =>
    ["schema-table-info", connectionId, schema, table] as const,
  tableDdl: (connectionId: string, schema: string, table: string) =>
    ["schema-table-ddl", connectionId, schema, table] as const,
};

function getSchemaService() {
  return container.resolve<ISchemaService>(SERVICE_NAMES.SCHEMA_SERVICE);
}

export function useIntrospect(connectionId: string | null) {
  return useQuery({
    queryKey: QUERY_KEYS.introspect(connectionId ?? ""),
    queryFn: () =>
      getSchemaService().introspect(connectionId!) as Promise<IntrospectResult>,
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
      getSchemaService().getTableInfo(
        connectionId!,
        schema!,
        table!,
      ) as Promise<TableInfo>,
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
    queryFn: () =>
      getSchemaService().getTableDdl(connectionId!, schema!, table!),
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
      }
    },
    onError: (err: unknown) => {
      console.error("[Schema] Cache invalidation failed:", err);
    },
  });
}
