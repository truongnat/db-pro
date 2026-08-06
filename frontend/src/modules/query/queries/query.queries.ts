import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { container } from "@/app/app.module";
import { SERVICE_NAMES, type IQueryService } from "@/commons/di/registry";
import { useQueryHistoryStore } from "@/commons/stores/query-history.store";

import { useQueryModuleStore } from "../state/query.store";
import type {
  ExplainPlan,
  QueryHistoryEntry,
  QueryResult,
  SavedQuery,
} from "../types/query.types";

const QUERY_KEYS = {
  history: (connectionId: string) => ["query-history", connectionId] as const,
  saved: (connectionId: string) => ["saved-queries", connectionId] as const,
};

function getQueryService() {
  return container.resolve<IQueryService>(SERVICE_NAMES.QUERY_SERVICE);
}

export function useExecuteQuery() {
  const qc = useQueryClient();
  const setStatus = useQueryModuleStore((s) => s.setStatus);
  const setError = useQueryModuleStore((s) => s.setError);
  const setResult = useQueryModuleStore((s) => s.setResult);

  return useMutation({
    mutationFn: ({ connectionId, sql }: { connectionId: string; sql: string }) =>
      getQueryService().execute(connectionId, sql) as Promise<QueryResult>,
    onMutate: () => {
      setStatus("running");
      setError(null);
    },
    onSuccess: (data, variables) => {
      setStatus("success");
      setResult(data);
      useQueryHistoryStore.getState().addEntry({
        id: crypto.randomUUID(),
        connectionId: variables.connectionId,
        sql: variables.sql,
        executedAt: new Date().toISOString(),
        durationMs: data.durationMs,
        rowCount: data.rowCount,
      });
      qc.invalidateQueries({
        queryKey: QUERY_KEYS.history(variables.connectionId),
      });
    },
    onError: (err: unknown) => {
      setStatus("error");
      setError(
        (err as { userMessage?: string }).userMessage ?? "Query execution failed",
      );
    },
  });
}

export function useExplainPlan() {
  const setExplainPlan = useQueryModuleStore((s) => s.setExplainPlan);
  const setActiveTab = useQueryModuleStore((s) => s.setActiveTab);

  return useMutation({
    mutationFn: ({ connectionId, sql }: { connectionId: string; sql: string }) =>
      getQueryService().explain(connectionId, sql) as Promise<ExplainPlan>,
    onSuccess: (data) => {
      setExplainPlan(data);
      setActiveTab("explain");
    },
  });
}

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
