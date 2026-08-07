import { apiInvoke } from "@/commons/utils/api";

import type {
  ExplainPlan,
  MultiQueryResult,
  QueryHistoryEntry,
  QueryResult,
  RunConfig,
  SavedQuery,
  SavedQueryFolder,
} from "../types/query.types";

export class QueryService {
  async execute(connectionId: string, sql: string): Promise<QueryResult> {
    return apiInvoke<QueryResult>("execute_query", {
      connectionId: connectionId,
      sql,
    });
  }

  async cancel(connectionId: string): Promise<void> {
    return apiInvoke<void>("cancel_query", {
      connectionId: connectionId,
    });
  }

  async executeMulti(connectionId: string, sql: string): Promise<MultiQueryResult> {
    return apiInvoke<MultiQueryResult>("execute_query_multi", {
      connectionId: connectionId,
      sql,
    });
  }

  async explain(connectionId: string, sql: string): Promise<ExplainPlan> {
    return apiInvoke<ExplainPlan>("explain_query", {
      connectionId: connectionId,
      sql,
    });
  }

  async getHistory(
    connectionId: string,
    limit?: number,
  ): Promise<QueryHistoryEntry[]> {
    return apiInvoke<QueryHistoryEntry[]>("get_query_history", {
      connectionId: connectionId,
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
      connectionId: connectionId,
      name,
      sql,
      folder,
    });
  }

  async listSaved(connectionId: string): Promise<SavedQuery[]> {
    return apiInvoke<SavedQuery[]>("list_saved_queries", {
      connectionId: connectionId,
    });
  }

  async deleteSaved(id: string): Promise<void> {
    return apiInvoke<void>("delete_saved_query", { id });
  }

  async createFolder(connectionId: string, name: string): Promise<SavedQueryFolder> {
    return apiInvoke<SavedQueryFolder>("create_folder", {
      connectionId: connectionId,
      name,
    });
  }

  async listFolders(connectionId: string): Promise<SavedQueryFolder[]> {
    return apiInvoke<SavedQueryFolder[]>("list_folders", {
      connectionId: connectionId,
    });
  }

  async deleteFolder(id: string): Promise<void> {
    return apiInvoke<void>("delete_folder", { id });
  }

  async saveRunConfig(
    connectionId: string,
    name: string,
    sql: string,
    timeoutMs: number,
    maxRows: number,
  ): Promise<RunConfig> {
    return apiInvoke<RunConfig>("save_run_config", {
      connectionId: connectionId,
      name,
      sql,
      timeoutMs: timeoutMs,
      maxRows: maxRows,
    });
  }

  async listRunConfigs(connectionId: string): Promise<RunConfig[]> {
    return apiInvoke<RunConfig[]>("list_run_configs", {
      connectionId: connectionId,
    });
  }

  async deleteRunConfig(id: string): Promise<void> {
    return apiInvoke<void>("delete_run_config", { id });
  }
}

export function createQueryService(): QueryService {
  return new QueryService();
}
