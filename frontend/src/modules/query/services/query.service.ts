import { apiInvoke } from "@/commons/utils/api";

import type {
  ExplainPlan,
  QueryHistoryEntry,
  QueryResult,
  SavedQuery,
} from "../types/query.types";

export class QueryService {
  async execute(connectionId: string, sql: string): Promise<QueryResult> {
    return apiInvoke<QueryResult>("execute_query", {
      connection_id: connectionId,
      sql,
    });
  }

  async explain(connectionId: string, sql: string): Promise<ExplainPlan> {
    return apiInvoke<ExplainPlan>("explain_query", {
      connection_id: connectionId,
      sql,
    });
  }

  async getHistory(
    connectionId: string,
    limit?: number,
  ): Promise<QueryHistoryEntry[]> {
    return apiInvoke<QueryHistoryEntry[]>("get_query_history", {
      connection_id: connectionId,
      limit,
    });
  }

  async save(
    connectionId: string,
    name: string,
    sql: string,
    folder?: string,
  ): Promise<SavedQuery> {
    return apiInvoke<SavedQuery>("save_query", {
      connection_id: connectionId,
      name,
      sql,
      folder,
    });
  }

  async listSaved(connectionId: string): Promise<SavedQuery[]> {
    return apiInvoke<SavedQuery[]>("list_saved_queries", {
      connection_id: connectionId,
    });
  }

  async deleteSaved(id: string): Promise<void> {
    return apiInvoke<void>("delete_saved_query", { id });
  }
}

export function createQueryService(): QueryService {
  return new QueryService();
}
