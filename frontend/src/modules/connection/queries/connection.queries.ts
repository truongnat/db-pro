import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { container } from "@/app/app.module";
import { useConnectionStore } from "@/commons/stores/connection.store";
import { useExplorerStore } from "@/commons/stores/explorer.store";
import { useRecentStore } from "@/commons/stores/recent.store";
import { SERVICE_NAMES, type IConnectionService } from "@/commons/di/registry";
import { useSchemaCatalogStore } from "@/modules/query/stores/schema-catalog.store";

import { useConnectionModuleStore } from "../state/connection.store";
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
    mutationFn: ({
      config,
      password,
      connectionId,
    }: {
      config: ConnectionConfig;
      password: string;
      connectionId?: string;
    }) => getConnectionService().test(config, password, connectionId),
  });
}

export function useConnect() {
  const qc = useQueryClient();
  const setStatus = useConnectionModuleStore((s) => s.setStatus);
  const setError = useConnectionModuleStore((s) => s.setError);
  const setExplorerConnection = useConnectionStore((s) => s.setExplorerConnection);
  const addRecentConnection = useRecentStore((s) => s.addRecentConnection);

  return useMutation({
    mutationFn: (id: string) => getConnectionService().connect(id),
    onMutate: (id) => {
      setStatus(id, "connecting");
    },
    onSuccess: (_, id) => {
      setStatus(id, "connected");
      setExplorerConnection(id);
      addRecentConnection(id);
      const expandedNodes = useExplorerStore.getState().expandedNodes;
      if (!expandedNodes.includes(`conn:${id}`)) {
        useExplorerStore.getState().toggleNode(`conn:${id}`);
      }
      qc.invalidateQueries({ queryKey: QUERY_KEYS.connections });
    },
    onError: (err: unknown, id) => {
      setStatus(id, "error");
      setError(id, (err as { userMessage?: string }).userMessage ?? "Connection failed");
    },
  });
}

export function useDisconnect() {
  const qc = useQueryClient();
  const setStatus = useConnectionModuleStore((s) => s.setStatus);
  const setError = useConnectionModuleStore((s) => s.setError);
  const clearStatus = useConnectionModuleStore((s) => s.clearStatus);
  const setExplorerConnection = useConnectionStore((s) => s.setExplorerConnection);
  const explorerConnectionId = useConnectionStore((s) => s.explorerConnectionId);

  return useMutation({
    mutationFn: (id: string) => getConnectionService().disconnect(id),
    onSuccess: (_, id) => {
      clearStatus(id);
      setStatus(id, "disconnected");
      if (explorerConnectionId === id) {
        setExplorerConnection(null);
      }
      qc.invalidateQueries({ queryKey: QUERY_KEYS.connections });
      useSchemaCatalogStore.getState().invalidateConnection(id);
    },
    onError: (err: unknown, id) => {
      setStatus(id, "error");
      setError(id, (err as { userMessage?: string }).userMessage ?? "Disconnect failed");
    },
  });
}
