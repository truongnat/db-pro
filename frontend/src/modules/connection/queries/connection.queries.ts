import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { container } from "@/app/app.module";
import { SERVICE_NAMES, type IConnectionService } from "@/commons/di/registry";

import type { Connection, ConnectionConfig } from "../types/connection.types";

const QUERY_KEYS = {
  connections: ["connections"] as const,
  connection: (id: string) => ["connections", id] as const,
};

function getConnectionService() {
  return container.resolve<IConnectionService>(SERVICE_NAMES.CONNECTION_SERVICE);
}

export function useConnectionList() {
  return useQuery({
    queryKey: QUERY_KEYS.connections,
    queryFn: () => getConnectionService().list() as Promise<Connection[]>,
  });
}

export function useCreateConnection() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ config, password }: { config: ConnectionConfig; password: string }) =>
      getConnectionService().create(config, password),
    onSuccess: () => qc.invalidateQueries({ queryKey: QUERY_KEYS.connections }),
  });
}

export function useUpdateConnection() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, config, password }: { id: string; config: ConnectionConfig; password?: string }) =>
      getConnectionService().update(id, config, password),
    onSuccess: () => qc.invalidateQueries({ queryKey: QUERY_KEYS.connections }),
  });
}

export function useDeleteConnection() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => getConnectionService().delete(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: QUERY_KEYS.connections }),
  });
}

export function useTestConnection() {
  return useMutation({
    mutationFn: ({ config, password }: { config: ConnectionConfig; password: string }) =>
      getConnectionService().test(config, password),
  });
}

export function useConnect() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => getConnectionService().connect(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: QUERY_KEYS.connections }),
  });
}

export function useDisconnect() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => getConnectionService().disconnect(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: QUERY_KEYS.connections }),
  });
}
