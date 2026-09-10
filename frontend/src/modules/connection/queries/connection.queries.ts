import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { container } from "@/app/app.module";
import { useConnectionStore } from "@/commons/stores/connection.store";
import { useExplorerStore } from "@/commons/stores/explorer.store";
import { useRecentStore } from "@/commons/stores/recent.store";
import { reconcileWorkspaceTabs } from "@/commons/stores/workspace.store";
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

const RECONNECT_CONCURRENCY = 3;
let sessionRestored = false;

// Test-only export to reset session restoration state between tests
export function __resetSessionRestored() {
  sessionRestored = false;
}

/**
 * Reconnect connections that were active in the previous session.
 * Only IDs that still exist in the saved connection list are attempted.
 * Failures are silent (no toast) — StatusDot in Explorer is sufficient.
 * This function only executes once per app startup to prevent double-connection
 * on subsequent query invalidations/refetches.
 */
function restoreSession(connections: Connection[]) {
  if (sessionRestored) return;
  sessionRestored = true;

  const { activeConnectionIds } = useConnectionStore.getState();
  if (activeConnectionIds.length === 0) return;

  const validIds = activeConnectionIds.filter((id) => connections.some((c) => c.id === id));
  // Clean up stale IDs that no longer exist.
  for (const staleId of activeConnectionIds) {
    if (!connections.some((c) => c.id === staleId)) {
      useConnectionStore.getState().removeActiveConnection(staleId);
    }
  }
  if (validIds.length === 0) return;

  const service = getConnectionService();
  const setStatus = useConnectionModuleStore.getState().setStatus;
  const setError = useConnectionModuleStore.getState().setError;
  const setExplorerConnection = useConnectionStore.getState().setExplorerConnection;
  const { explorerConnectionId } = useConnectionStore.getState();

  let index = 0;

  async function runNext(): Promise<void> {
    const i = index++;
    if (i >= validIds.length) return;
    const id = validIds[i];
    setStatus(id, "reconnecting");
    try {
      await service.connect(id);
      useConnectionModuleStore.getState().setStatus(id, "connected");
      // Restore explorer context for the first successfully reconnected
      // connection that matches the persisted explorer reference.
      if (explorerConnectionId === id) {
        setExplorerConnection(id);
        const expandedNodes = useExplorerStore.getState().expandedNodes;
        if (!expandedNodes.includes(`conn:${id}`)) {
          useExplorerStore.getState().expandNode(`conn:${id}`);
        }
      }
    } catch {
      useConnectionModuleStore.getState().setStatus(id, "error");
      setError(id, "Reconnect failed");
    }
    await runNext();
  }

  // Launch up to RECONNECT_CONCURRENCY parallel chains.
  const workers = Array.from({ length: Math.min(RECONNECT_CONCURRENCY, validIds.length) }, () =>
    runNext(),
  );
  Promise.allSettled(workers).then(() => {
    // Invalidate to refresh the connection list UI after reconnect.
    useConnectionStore.getState().setConnections(connections);
  });
}

export function useConnectionList() {
  return useQuery({
    queryKey: QUERY_KEYS.connections,
    queryFn: async () => {
      const connections = (await getConnectionService().list()) as Connection[];
      useConnectionStore.getState().setConnections(connections);
      // Connections are now loaded — reconcile persisted workspace tabs.
      reconcileWorkspaceTabs();
      // Restore previously active connections from the last session (one-shot).
      restoreSession(connections);
      return connections;
    },
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
    mutationFn: ({
      id,
      config,
      password,
    }: {
      id: string;
      config: ConnectionConfig;
      password?: string;
    }) => getConnectionService().update(id, config, password),
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
  const setActiveConnection = useConnectionStore((s) => s.setActiveConnection);
  const addRecentConnection = useRecentStore((s) => s.addRecentConnection);

  return useMutation({
    mutationFn: (id: string) => getConnectionService().connect(id),
    onMutate: (id) => {
      setStatus(id, "connecting");
    },
    onSuccess: (_, id) => {
      setStatus(id, "connected");
      setExplorerConnection(id);
      setActiveConnection(id);
      addRecentConnection(id);
      const expandedNodes = useExplorerStore.getState().expandedNodes;
      if (!expandedNodes.includes(`conn:${id}`)) {
        useExplorerStore.getState().expandNode(`conn:${id}`);
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
  const removeActiveConnection = useConnectionStore((s) => s.removeActiveConnection);
  const explorerConnectionId = useConnectionStore((s) => s.explorerConnectionId);

  return useMutation({
    mutationFn: (id: string) => getConnectionService().disconnect(id),
    onSuccess: (_, id) => {
      clearStatus(id);
      setStatus(id, "disconnected");
      removeActiveConnection(id);
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

/**
 * Duplicate an existing connection. Creates a new connection with the same
 * config but a new id and "(copy)" suffix on the name.
 */
export function useDuplicateConnection() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (id: string) => {
      const service = getConnectionService();
      const source = (await service.get(id)) as Connection | null;
      if (!source) throw new Error(`Connection ${id} not found`);
      const config: ConnectionConfig = {
        name: `${source.name} (copy)`,
        host: source.host,
        port: source.port,
        database: source.database,
        username: source.username,
        driver: source.driver,
        sslMode: source.sslMode,
        sshTunnel: source.sshTunnel,
        queryTimeoutMs: source.queryTimeoutMs ?? 30000,
        maxRows: source.maxRows ?? 500,
        color: source.color,
        tags: source.tags ? [...source.tags] : undefined,
        group: source.group,
        favorite: false,
        readonly: source.readonly,
      };
      return service.create(config, "");
    },
    onSuccess: () => qc.invalidateQueries({ queryKey: QUERY_KEYS.connections }),
  });
}

/**
 * Rename a connection by updating only the name field.
 */
export function useRenameConnection() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async ({ id, name }: { id: string; name: string }) => {
      const service = getConnectionService();
      const source = (await service.get(id)) as Connection | null;
      if (!source) throw new Error(`Connection ${id} not found`);
      const config: ConnectionConfig = {
        name,
        host: source.host,
        port: source.port,
        database: source.database,
        username: source.username,
        driver: source.driver,
        sslMode: source.sslMode,
        sshTunnel: source.sshTunnel,
        queryTimeoutMs: source.queryTimeoutMs ?? 30000,
        maxRows: source.maxRows ?? 500,
        color: source.color,
        tags: source.tags,
        group: source.group,
        favorite: source.favorite,
        readonly: source.readonly,
      };
      return service.update(id, config);
    },
    onSuccess: () => qc.invalidateQueries({ queryKey: QUERY_KEYS.connections }),
  });
}

/**
 * Toggle the favorite flag on a connection.
 */
export function useToggleFavorite() {
  const qc = useQueryClient();
  const toggleFavoriteLocal = useConnectionModuleStore((s) => s.toggleFavorite);

  return useMutation({
    mutationFn: async ({ id, favorite }: { id: string; favorite: boolean }) => {
      const service = getConnectionService();
      const source = (await service.get(id)) as Connection | null;
      if (!source) throw new Error(`Connection ${id} not found`);
      const config: ConnectionConfig = {
        name: source.name,
        host: source.host,
        port: source.port,
        database: source.database,
        username: source.username,
        driver: source.driver,
        sslMode: source.sslMode,
        sshTunnel: source.sshTunnel,
        queryTimeoutMs: source.queryTimeoutMs ?? 30000,
        maxRows: source.maxRows ?? 500,
        color: source.color,
        tags: source.tags,
        group: source.group,
        favorite,
        readonly: source.readonly,
      };
      return service.update(id, config);
    },
    onMutate: ({ id }) => {
      toggleFavoriteLocal(id);
    },
    onSuccess: () => qc.invalidateQueries({ queryKey: QUERY_KEYS.connections }),
  });
}

/**
 * Toggle the readonly flag on a connection.
 */
export function useToggleReadonly() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async ({ id, readonly }: { id: string; readonly: boolean }) => {
      const service = getConnectionService();
      const source = (await service.get(id)) as Connection | null;
      if (!source) throw new Error(`Connection ${id} not found`);
      const config: ConnectionConfig = {
        name: source.name,
        host: source.host,
        port: source.port,
        database: source.database,
        username: source.username,
        driver: source.driver,
        sslMode: source.sslMode,
        sshTunnel: source.sshTunnel,
        queryTimeoutMs: source.queryTimeoutMs ?? 30000,
        maxRows: source.maxRows ?? 500,
        color: source.color,
        tags: source.tags,
        group: source.group,
        favorite: source.favorite,
        readonly,
      };
      return service.update(id, config);
    },
    onSuccess: () => qc.invalidateQueries({ queryKey: QUERY_KEYS.connections }),
  });
}
