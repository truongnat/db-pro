import { beforeEach, describe, expect, it } from "vitest";

import { createDbObjectTab, createSchemaWorkspaceTab } from "@/commons/factories/tab-factories";
import { useConnectionStore } from "@/commons/stores/connection.store";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
import type { Connection, DriverType } from "@/modules/connection/types/connection.types";
import { useStagedChangesStore } from "@/modules/data-grid/state/staged-changes.store";

function connection(id: string, driver: DriverType): Connection {
  return {
    id,
    name: id,
    host: driver === "sqlite" ? "" : "localhost",
    port: driver === "sqlite" ? 0 : 5432,
    database: driver === "sqlite" ? "/tmp/db-pro-test.sqlite" : "app",
    username: driver === "sqlite" ? "" : "user",
    driver,
    sslMode: "disable",
    createdAt: "2026-08-17T00:00:00Z",
    updatedAt: "2026-08-17T00:00:00Z",
  };
}

function resetStores() {
  useWorkspaceStore.setState({ tabs: [], activeTabId: null, recentlyClosed: [] });
  useConnectionStore.setState({ connections: [] });
  useStagedChangesStore.getState().clearAll();
}

describe("workspace connection reassignment", () => {
  beforeEach(resetStores);

  it("preserves an explicit PostgreSQL schema named main when the orphan source is unknown", () => {
    const target = connection("pg-target", "postgres");
    useConnectionStore.setState({ connections: [target] });

    const tab = createDbObjectTab("missing-source", "main", "users", "table", "columns", false);
    useWorkspaceStore.setState({ tabs: [tab], activeTabId: tab.id });

    useWorkspaceStore.getState().reassignTabConnection(tab.id, target.id);

    const reassigned = useWorkspaceStore.getState().tabs[0];
    expect(reassigned.kind).toBe("db-object");
    if (reassigned.kind !== "db-object") return;
    expect(reassigned.connectionId).toBe(target.id);
    expect(reassigned.data.schema).toBe("main");
    expect(reassigned.resourceKey).toBe(`dbobj:main.users:${target.id}`);
  });

  it("refreshes a db-object title when SQLite main becomes PostgreSQL public", () => {
    const source = connection("sqlite-source", "sqlite");
    const target = connection("pg-target", "postgres");
    useConnectionStore.setState({ connections: [source, target] });

    const occupied = createDbObjectTab(target.id, "public", "users", "table", "columns", false);
    useWorkspaceStore.setState({ tabs: [occupied], activeTabId: occupied.id });
    const tab = createDbObjectTab(source.id, "main", "users", "table", "columns", false);
    expect(tab.title).toBe("main.users");
    useWorkspaceStore.setState({ tabs: [occupied, tab], activeTabId: tab.id });

    useWorkspaceStore.getState().reassignTabConnection(tab.id, target.id);

    const reassigned = useWorkspaceStore.getState().tabs.find((candidate) => candidate.id === tab.id);
    expect(reassigned?.kind).toBe("db-object");
    if (!reassigned || reassigned.kind !== "db-object") return;
    expect(reassigned.data.schema).toBe("public");
    expect(reassigned.title).toBe("public.users");
    expect(reassigned.resourceKey).toBe(`dbobj:public.users:${target.id}`);
  });

  it("maps a known SQLite default schema to PostgreSQL public and updates ER identity", () => {
    const source = connection("sqlite-source", "sqlite");
    const target = connection("pg-target", "postgres");
    useConnectionStore.setState({ connections: [source, target] });

    const tab = createSchemaWorkspaceTab(source.id, "main");
    useWorkspaceStore.setState({ tabs: [tab], activeTabId: tab.id });

    useWorkspaceStore.getState().reassignTabConnection(tab.id, target.id);

    const reassigned = useWorkspaceStore.getState().tabs[0];
    expect(reassigned.kind).toBe("schema-workspace");
    if (reassigned.kind !== "schema-workspace") return;
    expect(reassigned.data.schema).toBe("public");
    expect(reassigned.title).toBe("ER: public");
    expect(reassigned.resourceKey).toBe(`schema-ws:public:${target.id}`);
  });

  it("normalizes schema-workspace identity to SQLite main", () => {
    const source = connection("pg-source", "postgres");
    const target = connection("sqlite-target", "sqlite");
    useConnectionStore.setState({ connections: [source, target] });

    const tab = createSchemaWorkspaceTab(source.id, "analytics");
    useWorkspaceStore.setState({ tabs: [tab], activeTabId: tab.id });

    useWorkspaceStore.getState().reassignTabConnection(tab.id, target.id);

    const reassigned = useWorkspaceStore.getState().tabs[0];
    expect(reassigned.kind).toBe("schema-workspace");
    if (reassigned.kind !== "schema-workspace") return;
    expect(reassigned.data.schema).toBe("main");
    expect(reassigned.title).toBe("ER: main");
    expect(reassigned.resourceKey).toBe(`schema-ws:main:${target.id}`);
  });
});
