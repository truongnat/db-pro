import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { container } from "@/app/app.module";
import { SERVICE_NAMES, type IQueryService } from "@/commons/di/registry";

import {
  executeQuery,
  executeQueryMulti,
  cancelQuery,
  explainQuery,
} from "../runtime/query-runtime";
import type {
  ExplainPlan,
  MultiQueryResult,
  QueryHistoryEntry,
  QueryResult,
  RunConfig,
  SavedQuery,
  SavedQueryFolder,
} from "../types/query.types";

const QUERY_KEYS = {
  history: (connectionId: string) => ["query-history", connectionId] as const,
  saved: (connectionId: string) => ["saved-queries", connectionId] as const,
  folders: (connectionId: string) => ["saved-query-folders", connectionId] as const,
  runConfigs: (connectionId: string) => ["run-configs", connectionId] as const,
};

function getQueryService() {
  return container.resolve<IQueryService>(SERVICE_NAMES.QUERY_SERVICE);
}

/**
 * Execute a single query — delegates to the canonical runtime.
 *
 * The runtime owns ALL lifecycle behavior:
 *   - status management (running/success/error/cancelled)
 *   - stale response guard
 *   - result, timing, history recording
 *   - error normalization (QUERY_CANCELLED → cancelled)
 *
 * The hook adds TanStack Query cache invalidation on top.
 */
export function useExecuteQuery() {
  const qc = useQueryClient();

  return useMutation({
    mutationFn: ({ connectionId, sql, executionId, tabId }: {
      connectionId: string;
      sql: string;
      executionId: string;
      tabId: string;
    }) => executeQuery({ connectionId, sql, executionId, tabId }) as Promise<QueryResult>,
    onSuccess: (_data, variables) => {
      qc.invalidateQueries({ queryKey: QUERY_KEYS.history(variables.connectionId) });
    },
  });
}

/**
 * Cancel a running query — delegates to the canonical runtime.
 */
export function useCancelQuery() {
  return useMutation({
    mutationFn: ({ tabId, executionId }: { tabId: string; executionId: string }) =>
      cancelQuery({ tabId, executionId }),
  });
}

/**
 * Execute multiple statements — delegates to the canonical runtime.
 *
 * The runtime handles partial failure, history, timing, and all
 * lifecycle behavior. The hook adds TanStack cache invalidation.
 */
export function useExecuteQueryMulti() {
  const qc = useQueryClient();

  return useMutation({
    mutationFn: ({ connectionId, sql, executionId, tabId }: {
      connectionId: string;
      sql: string;
      executionId: string;
      tabId: string;
    }) => executeQueryMulti({ connectionId, sql, executionId, tabId }) as Promise<MultiQueryResult>,
    onSuccess: (_data, variables) => {
      qc.invalidateQueries({ queryKey: QUERY_KEYS.history(variables.connectionId) });
    },
  });
}

/**
 * Explain a query — delegates to the canonical runtime.
 *
 * The runtime populates the explain plan and activates the panel.
 */
export function useExplainPlan() {
  return useMutation({
    mutationFn: ({ connectionId, sql, tabId }: {
      connectionId: string;
      sql: string;
      tabId: string;
    }) => explainQuery({ connectionId, sql, tabId }) as Promise<ExplainPlan>,
  });
}

// ─── CRUD hooks (unchanged — these don't use the query runtime) ──

export function useQueryHistory(connectionId: string) {
  return useQuery({
    queryKey: QUERY_KEYS.history(connectionId),
    queryFn: () =>
      getQueryService().getHistory(connectionId) as Promise<QueryHistoryEntry[]>,
    enabled: !!connectionId,
  });
}

export function useSaveQuery() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({
      connectionId,
      name,
      sql,
      folder,
    }: {
      connectionId: string;
      name: string;
      sql: string;
      folder?: string;
    }) => getQueryService().save(connectionId, name, sql, folder) as Promise<SavedQuery>,
    onSuccess: (_, variables) => {
      qc.invalidateQueries({ queryKey: QUERY_KEYS.saved(variables.connectionId) });
    },
  });
}

export function useListSavedQueries(connectionId: string) {
  return useQuery({
    queryKey: QUERY_KEYS.saved(connectionId),
    queryFn: () =>
      getQueryService().listSaved(connectionId) as Promise<SavedQuery[]>,
    enabled: !!connectionId,
  });
}

export function useDeleteSavedQuery() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, connectionId }: { id: string; connectionId: string }) =>
      getQueryService().deleteSaved(id),
    onSuccess: (_, variables) => {
      qc.invalidateQueries({ queryKey: QUERY_KEYS.saved(variables.connectionId) });
    },
  });
}

export function useListFolders(connectionId: string) {
  return useQuery({
    queryKey: QUERY_KEYS.folders(connectionId),
    queryFn: () =>
      getQueryService().listFolders(connectionId) as Promise<SavedQueryFolder[]>,
    enabled: !!connectionId,
  });
}

export function useCreateFolder() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ connectionId, name }: { connectionId: string; name: string }) =>
      getQueryService().createFolder(connectionId, name) as Promise<SavedQueryFolder>,
    onSuccess: (_, variables) => {
      qc.invalidateQueries({ queryKey: QUERY_KEYS.folders(variables.connectionId) });
    },
  });
}

export function useDeleteFolder() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, connectionId }: { id: string; connectionId: string }) =>
      getQueryService().deleteFolder(id),
    onSuccess: (_, variables) => {
      qc.invalidateQueries({ queryKey: QUERY_KEYS.folders(variables.connectionId) });
    },
  });
}

export function useListRunConfigs(connectionId: string) {
  return useQuery({
    queryKey: QUERY_KEYS.runConfigs(connectionId),
    queryFn: () =>
      getQueryService().listRunConfigs(connectionId) as Promise<RunConfig[]>,
    enabled: !!connectionId,
  });
}

export function useSaveRunConfig() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({
      connectionId,
      name,
      sql,
      timeoutMs,
      maxRows,
    }: {
      connectionId: string;
      name: string;
      sql: string;
      timeoutMs: number;
      maxRows: number;
    }) =>
      getQueryService().saveRunConfig(connectionId, name, sql, timeoutMs, maxRows) as Promise<RunConfig>,
    onSuccess: (_, variables) => {
      qc.invalidateQueries({ queryKey: QUERY_KEYS.runConfigs(variables.connectionId) });
    },
  });
}

export function useDeleteRunConfig() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, connectionId }: { id: string; connectionId: string }) =>
      getQueryService().deleteRunConfig(id),
    onSuccess: (_, variables) => {
      qc.invalidateQueries({ queryKey: QUERY_KEYS.runConfigs(variables.connectionId) });
    },
  });
}

/**
 * Rename a saved query by deleting and re-saving with a new name.
 * Note: this changes the ID and created_at, which is acceptable for P11.
 */
export function useRenameSavedQuery() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async ({
      id,
      connectionId,
      newName,
    }: {
      id: string;
      connectionId: string;
      newName: string;
    }) => {
      const queries =
        (qc.getQueryData(QUERY_KEYS.saved(connectionId)) as SavedQuery[] | undefined) ?? [];
      const existing = queries.find((q) => q.id === id);
      if (!existing) throw new Error("Saved query not found");
      await getQueryService().deleteSaved(id);
      return getQueryService().save(
        connectionId,
        newName,
        existing.sql,
        existing.folder,
      ) as Promise<SavedQuery>;
    },
    onSuccess: (_, variables) => {
      qc.invalidateQueries({ queryKey: QUERY_KEYS.saved(variables.connectionId) });
    },
  });
}

/**
 * Duplicate a saved query with a "(copy)" suffix.
 */
export function useDuplicateSavedQuery() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async ({
      id,
      connectionId,
    }: {
      id: string;
      connectionId: string;
    }) => {
      const queries =
        (qc.getQueryData(QUERY_KEYS.saved(connectionId)) as SavedQuery[] | undefined) ?? [];
      const existing = queries.find((q) => q.id === id);
      if (!existing) throw new Error("Saved query not found");
      return getQueryService().save(
        connectionId,
        `${existing.name} (copy)`,
        existing.sql,
        existing.folder,
      ) as Promise<SavedQuery>;
    },
    onSuccess: (_, variables) => {
      qc.invalidateQueries({ queryKey: QUERY_KEYS.saved(variables.connectionId) });
    },
  });
}
