import { describe, expect, it } from "vitest";

import { buildQuickOpenIndex } from "@/commons/services/quick-open-index";
import type { Connection } from "@/modules/connection/types/connection.types";
import type { WorkspaceTab } from "@/commons/types/workspace.types";
import type { ConnectionCatalog } from "@/modules/query/stores/schema-catalog.store";

const conns: Connection[] = [
  {
    id: "conn-1",
    name: "Local",
    host: "localhost",
    port: 5432,
    database: "app",
    username: "dev",
    driver: "postgres",
    sslMode: "disable",
    createdAt: "",
    updatedAt: "",
  },
];

const catalog: ConnectionCatalog = {
  schemas: [{ name: "public" }, { name: "auth" }],
  objects: [
    { name: "client", schema: "public", rowCount: 1, kind: "table" },
    { name: "client_summary", schema: "reporting", rowCount: null, kind: "view" },
  ],
  columnsByTable: new Map(),
  columnsLoaded: new Set(),
  columnsLoading: new Map(),
};

function tab(overrides: Partial<WorkspaceTab> = {}): WorkspaceTab {
  return {
    id: "tab-1",
    kind: "db-object",
    title: "client",
    connectionId: "conn-1",
    resourceKey: "dbobj:public.client:conn-1",
    dirty: false,
    pinned: false,
    preview: true,
    order: 1,
    data: { schema: "public", objectName: "client", objectType: "table", activeSection: "columns" },
    ...overrides,
  } as WorkspaceTab;
}

describe("buildQuickOpenIndex", () => {
  it("builds tab items from workspace tabs", () => {
    const items = buildQuickOpenIndex({
      connections: conns,
      catalogs: new Map(),
      tabs: [tab()],
    });
    const tabItems = items.filter((i) => i.kind === "tab");
    expect(tabItems).toHaveLength(1);
    expect(tabItems[0]).toMatchObject({ kind: "tab", title: "client", connectionName: "Local" });
  });

  it("builds db-object items from catalog with qualified search text", () => {
    const items = buildQuickOpenIndex({
      connections: conns,
      catalogs: new Map([["conn-1", catalog]]),
      tabs: [],
    });
    const dbItems = items.filter((i) => i.kind === "db-object");
    expect(dbItems).toHaveLength(2);
    const client = dbItems.find((i) => i.objectName === "client");
    expect(client).toBeDefined();
    expect(client?.searchText).toContain("public.client");
    expect(client?.connectionName).toBe("Local");
    const view = dbItems.find((i) => i.objectName === "client_summary");
    expect(view?.objectType).toBe("view");
  });

  it("builds schema items from catalog schemas", () => {
    const items = buildQuickOpenIndex({
      connections: conns,
      catalogs: new Map([["conn-1", catalog]]),
      tabs: [],
    });
    const schemaItems = items.filter((i) => i.kind === "schema");
    expect(schemaItems.map((s) => s.schema).sort()).toEqual(["auth", "public"]);
  });

  it("builds connection items with driver/database search text", () => {
    const items = buildQuickOpenIndex({
      connections: conns,
      catalogs: new Map(),
      tabs: [],
    });
    const connItems = items.filter((i) => i.kind === "connection");
    expect(connItems).toHaveLength(1);
    expect(connItems[0].searchText).toContain("app");
    expect(connItems[0].searchText).toContain("postgres");
  });

  it("uses connectionId as name fallback when unknown", () => {
    const items = buildQuickOpenIndex({
      connections: [],
      catalogs: new Map([["conn-x", catalog]]),
      tabs: [tab({ connectionId: "conn-x" })],
    });
    const tabItem = items.find((i) => i.kind === "tab");
    expect(tabItem?.connectionName).toBe("conn-x");
  });
});
