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

  it("introspect calls introspect with connection_id", async () => {
    mockInvoke.mockResolvedValueOnce({ schemas: [], tables: [], columns: [], primaryKeys: [], indexes: [], foreignKeys: [], views: [] });
    await service.introspect("conn-1");
    expect(mockInvoke).toHaveBeenCalledWith("introspect", {
      connection_id: "conn-1",
      force_refresh: undefined,
    });
  });

  it("introspect passes force_refresh when specified", async () => {
    mockInvoke.mockResolvedValueOnce({ schemas: [], tables: [], columns: [], primaryKeys: [], indexes: [], foreignKeys: [], views: [] });
    await service.introspect("conn-1", true);
    expect(mockInvoke).toHaveBeenCalledWith("introspect", {
      connection_id: "conn-1",
      force_refresh: true,
    });
  });

  it("getTableInfo calls get_table_info with all args", async () => {
    mockInvoke.mockResolvedValueOnce({ table: {}, columns: [], primaryKey: null, indexes: [], foreignKeys: [] });
    await service.getTableInfo("conn-1", "public", "users");
    expect(mockInvoke).toHaveBeenCalledWith("get_table_info", {
      connection_id: "conn-1",
      schema: "public",
      table: "users",
    });
  });

  it("getTableDdl calls get_table_ddl with all args", async () => {
    mockInvoke.mockResolvedValueOnce("CREATE TABLE ...");
    const result = await service.getTableDdl("conn-1", "public", "users");
    expect(mockInvoke).toHaveBeenCalledWith("get_table_ddl", {
      connection_id: "conn-1",
      schema: "public",
      table: "users",
    });
    expect(result).toBe("CREATE TABLE ...");
  });

  it("invalidateCache calls invalidate_cache with connection_id", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);
    await service.invalidateCache("conn-1");
    expect(mockInvoke).toHaveBeenCalledWith("invalidate_cache", {
      connection_id: "conn-1",
    });
  });
});
