import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { container } from "@/app/app.module";
import { SERVICE_NAMES, type IDataGridService } from "@/commons/di/registry";

import type {
  FetchRowsRequest,
  FetchRowsResult,
  MutateRowRequest,
  MutateRowResult,
} from "../types/data-grid.types";

const QUERY_KEYS = {
  tableRows: (
    connectionId: string,
    schema: string,
    table: string,
    page: number,
    pageSize: number,
    filters: string,
    sorts: string,
  ) => ["data-grid-rows", connectionId, schema, table, page, pageSize, filters, sorts] as const,
};

function getDataGridService() {
  return container.resolve<IDataGridService>(SERVICE_NAMES.DATA_GRID_SERVICE);
}

export function useTableRows(connectionId: string | null, request: FetchRowsRequest | null) {
  const enabled = !!connectionId && !!request && !!request.table;
  const filtersKey = JSON.stringify(request?.filters ?? []);
  const sortsKey = JSON.stringify(request?.sorts ?? []);

  return useQuery({
    queryKey: QUERY_KEYS.tableRows(
      connectionId ?? "",
      request?.schema ?? "",
      request?.table ?? "",
      request?.page ?? 1,
      request?.pageSize ?? 50,
      filtersKey,
      sortsKey,
    ),
    queryFn: () =>
      getDataGridService().fetchRows(connectionId!, request!) as Promise<FetchRowsResult>,
    enabled,
  });
}

export function useInsertRow(connectionId: string | null, request: FetchRowsRequest | null) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (req: MutateRowRequest) =>
      getDataGridService().insertRow(connectionId!, req) as Promise<MutateRowResult>,
    onSuccess: () => {
      if (connectionId && request) {
        invalidateRows(qc, connectionId, request);
      }
    },
    onError: (err: unknown) => {
      console.error("[DataGrid] Insert failed:", err);
    },
  });
}

export function useUpdateRow(connectionId: string | null, request: FetchRowsRequest | null) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (req: MutateRowRequest) =>
      getDataGridService().updateRow(connectionId!, req) as Promise<MutateRowResult>,
    onSuccess: () => {
      if (connectionId && request) {
        invalidateRows(qc, connectionId, request);
      }
    },
    onError: (err: unknown) => {
      console.error("[DataGrid] Update failed:", err);
    },
  });
}

export function useDeleteRow(connectionId: string | null, request: FetchRowsRequest | null) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (req: MutateRowRequest) =>
      getDataGridService().deleteRow(connectionId!, req) as Promise<MutateRowResult>,
    onSuccess: () => {
      if (connectionId && request) {
        invalidateRows(qc, connectionId, request);
      }
    },
    onError: (err: unknown) => {
      console.error("[DataGrid] Delete failed:", err);
    },
  });
}

function invalidateRows(
  qc: ReturnType<typeof useQueryClient>,
  connectionId: string,
  request: FetchRowsRequest,
) {
  qc.invalidateQueries({
    queryKey: ["data-grid-rows", connectionId, request.schema, request.table],
  });
}
