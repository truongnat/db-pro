import { describe, expect, it, vi } from "vitest";

const { mockInvoke } = vi.hoisted(() => ({ mockInvoke: vi.fn() }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mockInvoke,
}));

import { SchemaService } from "../services/schema.service";

const service = new SchemaService();

describe("SchemaService", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
  });

  it("introspect calls introspect with connectionId", async () => {
    mockInvoke.mockResolvedValueOnce({
      schemas: [],
      tables: [],
      columns: [],
      primaryKeys: [],
      indexes: [],
      foreignKeys: [],
      views: [],
    });
    await service.introspect("conn-1");
    expect(mockInvoke).toHaveBeenCalledWith("introspect", {
      connectionId: "conn-1",
      forceRefresh: undefined,
    });
  });

  it("introspect passes forceRefresh when specified", async () => {
    mockInvoke.mockResolvedValueOnce({
      schemas: [],
      tables: [],
      columns: [],
      primaryKeys: [],
      indexes: [],
      foreignKeys: [],
      views: [],
    });
    await service.introspect("conn-1", true);
    expect(mockInvoke).toHaveBeenCalledWith("introspect", {
      connectionId: "conn-1",
      forceRefresh: true,
    });
  });

  it("getTableInfo calls get_table_info with all args", async () => {
    mockInvoke.mockResolvedValueOnce({
      table: {},
      columns: [],
      primaryKey: null,
      indexes: [],
      foreignKeys: [],
    });
    await service.getTableInfo("conn-1", "public", "users");
    expect(mockInvoke).toHaveBeenCalledWith("get_table_info", {
      connectionId: "conn-1",
      schema: "public",
      table: "users",
    });
  });

  it("getTableDdl calls get_table_ddl with all args", async () => {
    mockInvoke.mockResolvedValueOnce("CREATE TABLE ...");
    const result = await service.getTableDdl("conn-1", "public", "users");
    expect(mockInvoke).toHaveBeenCalledWith("get_table_ddl", {
      connectionId: "conn-1",
      schema: "public",
      table: "users",
    });
    expect(result).toBe("CREATE TABLE ...");
  });

  it("invalidateCache calls invalidate_cache with connectionId", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);
    await service.invalidateCache("conn-1");
    expect(mockInvoke).toHaveBeenCalledWith("invalidate_cache", {
      connectionId: "conn-1",
    });
  });
});
