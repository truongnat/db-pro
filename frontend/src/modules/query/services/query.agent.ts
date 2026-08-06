import type {
  ExplainPlan,
  MultiQueryResult,
  QueryHistoryEntry,
  QueryResult,
  Row,
  RunConfig,
  SavedQuery,
  SavedQueryFolder,
} from "../types/query.types";

export class MockQueryService {
  private history: QueryHistoryEntry[] = [];
  private saved: SavedQuery[] = [];
  private runConfigs: RunConfig[] = [];
  private nextId = 1;

  async execute(_connectionId: string, sql: string): Promise<QueryResult> {
    const rows: Row[] = [
      [
        { type: "int64", value: 1 },
        { type: "text", value: "Alice" },
        { type: "text", value: "alice@example.com" },
      ],
      [
        { type: "int64", value: 2 },
        { type: "text", value: "Bob" },
        { type: "null" },
      ],
    ];

    const entry: QueryHistoryEntry = {
      id: String(this.nextId++),
      connectionId: _connectionId,
      sql,
      executedAt: new Date().toISOString(),
      durationMs: 42,
      rowCount: 2,
    };
    this.history.unshift(entry);

    return {
      columns: [
        { name: "id", dataType: "INTEGER", nullable: false },
        { name: "name", dataType: "TEXT", nullable: false },
        { name: "email", dataType: "TEXT", nullable: true },
      ],
      rows,
      rowCount: 2,
      durationMs: 42,
    };
  }

  async cancel(_connectionId: string): Promise<void> {
    // mock: no-op
  }

  async executeMulti(_connectionId: string, _sql: string): Promise<MultiQueryResult> {
    const result = await this.execute(_connectionId, "SELECT 1");
    return { results: [result], totalDurationMs: 84 };
  }

  async explain(_connectionId: string, _sql: string): Promise<ExplainPlan> {
    return {
      "Plan": {
        "Node Type": "Seq Scan",
        "Relation Name": "users",
        "Startup Cost": 0,
        "Total Cost": 25.5,
        "Plan Rows": 155,
        "Plan Width": 32,
      },
    };
  }

  async getHistory(
    _connectionId: string,
    limit?: number,
  ): Promise<QueryHistoryEntry[]> {
    return limit ? this.history.slice(0, limit) : this.history;
  }

  async save(
    connectionId: string,
    name: string,
    sql: string,
    folder?: string,
  ): Promise<SavedQuery> {
    const entry: SavedQuery = {
      id: String(this.nextId++),
      connectionId,
      name,
      sql,
      folder,
      createdAt: new Date().toISOString(),
    };
    this.saved.push(entry);
    return entry;
  }

  async listSaved(_connectionId: string): Promise<SavedQuery[]> {
    return this.saved;
  }

  async deleteSaved(id: string): Promise<void> {
    this.saved = this.saved.filter((s) => s.id !== id);
  }

  async createFolder(connectionId: string, name: string): Promise<SavedQueryFolder> {
    const folder: SavedQueryFolder = {
      id: String(this.nextId++),
      connectionId,
      name,
      createdAt: new Date().toISOString(),
    };
    return folder;
  }

  async listFolders(_connectionId: string): Promise<SavedQueryFolder[]> {
    return [];
  }

  async deleteFolder(_id: string): Promise<void> {
    // mock: no-op
  }

  async saveRunConfig(
    connectionId: string,
    name: string,
    sql: string,
    timeoutMs: number,
    maxRows: number,
  ): Promise<RunConfig> {
    const config: RunConfig = {
      id: String(this.nextId++),
      connectionId,
      name,
      sql,
      timeoutMs,
      maxRows,
      createdAt: new Date().toISOString(),
    };
    this.runConfigs.push(config);
    return config;
  }

  async listRunConfigs(_connectionId: string): Promise<RunConfig[]> {
    return this.runConfigs;
  }

  async deleteRunConfig(id: string): Promise<void> {
    this.runConfigs = this.runConfigs.filter((c) => c.id !== id);
  }
}
