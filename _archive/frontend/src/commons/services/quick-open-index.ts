import type { QuickOpenItem } from "@/commons/types/quick-open.types";
import type { Connection } from "@/modules/connection/types/connection.types";
import type { WorkspaceTab } from "@/commons/types/workspace.types";
import type { ConnectionCatalog } from "@/modules/query/stores/schema-catalog.store";

export interface QuickOpenIndexInput {
  connections: Connection[];
  catalogs: Map<string, ConnectionCatalog>;
  tabs: WorkspaceTab[];
}

function connectionName(id: string | null, map: Map<string, string>): string {
  if (!id) return "";
  return map.get(id) ?? id;
}

export function buildQuickOpenIndex(input: QuickOpenIndexInput): QuickOpenItem[] {
  const { connections, catalogs, tabs } = input;
  const connNameMap = new Map(connections.map((c) => [c.id, c.name]));
  const items: QuickOpenItem[] = [];

  for (const tab of tabs) {
    const name = connectionName(tab.connectionId, connNameMap);
    const searchParts: string[] = [tab.title, name];
    if (tab.kind === "db-object") {
      searchParts.push(tab.data.schema, tab.data.objectName);
    }
    items.push({
      kind: "tab",
      tabId: tab.id,
      title: tab.title,
      connectionId: tab.connectionId,
      connectionName: name,
      resourceKey: tab.resourceKey,
      searchText: searchParts.join(" "),
    });
  }

  for (const [connId, catalog] of catalogs) {
    const name = connectionName(connId, connNameMap);
    for (const schema of catalog.schemas) {
      const qualified = `${schema.name}.`;
      items.push({
        kind: "schema",
        connectionId: connId,
        connectionName: name,
        schema: schema.name,
        resourceKey: `schema:${schema.name}:${connId}`,
        searchText: `${schema.name} ${qualified} ${name}`,
      });
    }
    for (const obj of catalog.objects) {
      const qualified = `${obj.schema}.${obj.name}`;
      items.push({
        kind: "db-object",
        connectionId: connId,
        connectionName: name,
        schema: obj.schema,
        objectName: obj.name,
        objectType: obj.kind,
        resourceKey: `dbobj:${obj.schema}.${obj.name}:${connId}`,
        searchText: `${obj.name} ${qualified} ${obj.schema} ${name}`,
      });
    }
  }

  for (const conn of connections) {
    items.push({
      kind: "connection",
      connectionId: conn.id,
      connectionName: conn.name,
      resourceKey: `connection:${conn.id}`,
      searchText: `${conn.name} ${conn.driver} ${conn.database}`,
    });
  }

  return items;
}
