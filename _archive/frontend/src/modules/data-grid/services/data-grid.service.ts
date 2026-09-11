import { apiInvoke } from "@/commons/utils/api";

import type {
  FetchRowsRequest,
  FetchRowsResult,
  MutateRowRequest,
  MutateRowResult,
} from "../types/data-grid.types";

export class DataGridService {
  async fetchRows(connectionId: string, request: FetchRowsRequest): Promise<FetchRowsResult> {
    return apiInvoke<FetchRowsResult>("fetch_table_rows", {
      connectionId: connectionId,
      request: {
        schema: request.schema,
        table: request.table,
        filters: request.filters.map((f) => ({
          column: f.column,
          op: f.op,
          value: f.value,
        })),
        sorts: request.sorts.map((s) => ({
          column: s.column,
          direction: s.direction,
        })),
        page: request.page,
        pageSize: request.pageSize,
      },
    });
  }

  async insertRow(connectionId: string, request: MutateRowRequest): Promise<MutateRowResult> {
    return apiInvoke<MutateRowResult>("insert_table_row", {
      connectionId: connectionId,
      request: {
        schema: request.schema,
        table: request.table,
        columns: request.columns,
        values: request.values,
      },
    });
  }

  async updateRow(connectionId: string, request: MutateRowRequest): Promise<MutateRowResult> {
    return apiInvoke<MutateRowResult>("update_table_row", {
      connectionId: connectionId,
      request: {
        schema: request.schema,
        table: request.table,
        columns: request.columns,
        values: request.values,
        pkColumns: request.pkColumns,
        pkValues: request.pkValues,
      },
    });
  }

  async deleteRow(connectionId: string, request: MutateRowRequest): Promise<MutateRowResult> {
    return apiInvoke<MutateRowResult>("delete_table_row", {
      connectionId: connectionId,
      request: {
        schema: request.schema,
        table: request.table,
        columns: request.columns,
        values: request.values,
        pkColumns: request.pkColumns,
        pkValues: request.pkValues,
      },
    });
  }
}

export function createDataGridService(): DataGridService {
  return new DataGridService();
}
