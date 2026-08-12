export interface AgentMessage {
  id: string;
  role: "user" | "assistant";
  content: string;
  sql?: string;
  timestamp: number;
}

export interface AgentConfig {
  apiEndpoint: string;
  apiKey: string;
  model: string;
}

export interface SchemaContext {
  tables: { name: string; schema: string; rowCount: number | null }[];
  columns: Map<
    string,
    { name: string; dataType: string; nullable: boolean; isPrimaryKey: boolean }[]
  >;
  foreignKeys: {
    fromTable: string;
    fromColumn: string;
    toTable: string;
    toColumn: string;
    schema: string;
  }[];
  connectionName: string | null;
  activeSchema: string | null;
  activeTable: string | null;
}
