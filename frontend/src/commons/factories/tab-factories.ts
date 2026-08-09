import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import { useConnectionStore } from "@/commons/stores/connection.store";
import type {
  DbObjectSection,
  DbObjectTabData,
  QueryContext,
  QueryTabData,
  WorkspaceTab,
} from "@/commons/types/workspace.types";

function nextOrder(): number {
  const tabs = useWorkspaceStore.getState().tabs;
  if (tabs.length === 0) return 1;
  return Math.max(...tabs.map((t) => t.order)) + 1;
}

function nextQueryTitle(connectionId: string): string {
  const tabs = useWorkspaceStore.getState().tabs;
  const usedNumbers = new Set<number>();
  for (const t of tabs) {
    if (t.kind === "query" && t.connectionId === connectionId) {
      const match = t.title.match(/^Query (\d+)$/);
      if (match) usedNumbers.add(parseInt(match[1], 10));
    }
  }
  let n = 1;
  while (usedNumbers.has(n)) n++;
  return `Query ${n}`;
}

function defaultQueryData(): QueryTabData {
  return {
    context: { database: null, schema: null },
    sql: "",
    status: "idle",
    error: null,
    result: null,
    explainPlan: null,
    sort: { column: null, direction: null },
    multiResults: null,
    multiResultIndex: 0,
    activePanel: "results",
    timing: null,
    executionStartedAt: null,
    activeExecutionId: null,
  };
}

/**
 * Deterministic collision resolution for db-object tab titles:
 * 1. Try `objectName` (e.g. "users")
 * 2. If collision: try `schema.objectName` (e.g. "audit.users")
 * 3. If still collision: "schema.objectName (connectionName)"
 */
function deduplicateTabTitle(objectName: string, schema: string, connectionId: string): string {
  const tabs = useWorkspaceStore.getState().tabs;
  const connections = useConnectionStore.getState().connections;
  const connName = connections.find((c) => c.id === connectionId)?.name ?? connectionId;

  // Level 1: bare object name
  const bare = objectName;
  if (!tabs.some((t) => t.title === bare)) return bare;

  // Level 2: schema.objectName
  const qualified = `${schema}.${objectName}`;
  if (!tabs.some((t) => t.title === qualified)) return qualified;

  // Level 3: fully qualified with connection
  return `${connName} \u00B7 ${qualified}`;
}

export interface CreateQueryTabOptions {
  title?: string;
  sql?: string;
  context?: QueryContext;
}

export function createQueryTab(
  connectionId: string,
  options?: CreateQueryTabOptions,
): WorkspaceTab & { kind: "query" } {
  const id = crypto.randomUUID();
  const title = options?.title ?? nextQueryTitle(connectionId);
  const context = options?.context ?? { database: null, schema: null };
  return {
    id,
    kind: "query",
    title,
    connectionId,
    resourceKey: `query:${id}`,
    dirty: false,
    pinned: false,
    preview: false,
    order: nextOrder(),
    data: {
      ...defaultQueryData(),
      context,
      sql: options?.sql ?? "",
    },
  };
}

export function createDbObjectTab(
  connectionId: string,
  schema: string,
  objectName: string,
  objectType: DbObjectTabData["objectType"],
  initialSection: DbObjectSection = "columns",
  preview = true,
): WorkspaceTab & { kind: "db-object" } {
  const title = deduplicateTabTitle(objectName, schema, connectionId);
  return {
    id: crypto.randomUUID(),
    kind: "db-object",
    title,
    connectionId,
    resourceKey: `dbobj:${schema}.${objectName}:${connectionId}`,
    dirty: false,
    pinned: false,
    preview,
    order: nextOrder(),
    data: {
      schema,
      objectName,
      objectType,
      activeSection: initialSection,
    } satisfies DbObjectTabData,
  };
}
