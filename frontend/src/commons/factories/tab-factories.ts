import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import type {
  QueryTabData,
  SchemaObjectTabData,
  TableDataTabData,
  WorkspaceTab,
} from "@/commons/types/workspace.types";

function nextOrder(): number {
  const tabs = useWorkspaceStore.getState().tabs;
  if (tabs.length === 0) return 1;
  return Math.max(...tabs.map((t) => t.order)) + 1;
}

function nextQueryTitle(connectionId: string): string {
  const tabs = useWorkspaceStore.getState().tabs;
  const queryCount = tabs.filter(
    (t) => t.kind === "query" && t.connectionId === connectionId,
  ).length;
  return `Query ${queryCount + 1}`;
}

function defaultQueryData(): QueryTabData {
  return {
    sql: "",
    status: "idle",
    error: null,
    result: null,
    explainPlan: null,
    sort: { column: null, direction: null },
    multiResults: null,
    multiResultIndex: 0,
  };
}

export function createQueryTab(connectionId: string, title?: string, sql?: string): WorkspaceTab & { kind: "query" } {
  const id = crypto.randomUUID();
  return {
    id,
    kind: "query",
    title: title ?? nextQueryTitle(connectionId),
    connectionId,
    resourceKey: `query:${id}`,
    dirty: false,
    pinned: false,
    preview: false,
    order: nextOrder(),
    data: {
      ...defaultQueryData(),
      sql: sql ?? "",
    },
  };
}

export function createTableDataTab(
  connectionId: string,
  schema: string,
  table: string,
): WorkspaceTab & { kind: "table-data" } {
  return {
    id: crypto.randomUUID(),
    kind: "table-data",
    title: table,
    connectionId,
    resourceKey: `table:${schema}.${table}:${connectionId}`,
    dirty: false,
    pinned: false,
    preview: false,
    order: nextOrder(),
    data: { schema, table } satisfies TableDataTabData,
  };
}

export function createSchemaObjectTab(
  connectionId: string,
  schema: string,
  objectName: string,
  objectType: SchemaObjectTabData["objectType"],
): WorkspaceTab & { kind: "schema-object" } {
  return {
    id: crypto.randomUUID(),
    kind: "schema-object",
    title: objectName,
    connectionId,
    resourceKey: `object:${schema}.${objectName}:${connectionId}`,
    dirty: false,
    pinned: false,
    preview: true,
    order: nextOrder(),
    data: { schema, objectName, objectType } satisfies SchemaObjectTabData,
  };
}
