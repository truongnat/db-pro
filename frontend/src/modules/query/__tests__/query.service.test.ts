import { describe, expect, it, vi } from "vitest";

const { mockInvoke } = vi.hoisted(() => ({ mockInvoke: vi.fn() }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mockInvoke,
}));

import { QueryService } from "../services/query.service";

const service = new QueryService();

describe("QueryService", () => {
  it("execute calls execute_query with connection_id and sql", async () => {
    mockInvoke.mockResolvedValueOnce({
      columns: [],
      rows: [],
      rowCount: 0,
      durationMs: 10,
    });

    await service.execute("conn-1", "SELECT 1");

    expect(mockInvoke).toHaveBeenCalledWith("execute_query", {
      connectionId: "conn-1",
      sql: "SELECT 1",
      executionId: undefined,
      database: null,
      schema: null,
    });
  });

  it("explain calls explain_query with connection_id and sql", async () => {
    mockInvoke.mockResolvedValueOnce({ Plan: {} });

    await service.explain("conn-1", "SELECT 1");

    expect(mockInvoke).toHaveBeenCalledWith("explain_query", {
      connectionId: "conn-1",
      sql: "SELECT 1",
    });
  });

  it("getHistory calls get_query_history with connection_id and limit", async () => {
    mockInvoke.mockResolvedValueOnce([]);

    await service.getHistory("conn-1", 50);

    expect(mockInvoke).toHaveBeenCalledWith("get_query_history", {
      connectionId: "conn-1",
      limit: 50,
    });
  });

  it("getHistory works without limit", async () => {
    mockInvoke.mockResolvedValueOnce([]);

    await service.getHistory("conn-1");

    expect(mockInvoke).toHaveBeenCalledWith("get_query_history", {
      connectionId: "conn-1",
      limit: undefined,
    });
  });

  it("save calls save_query with all parameters", async () => {
    mockInvoke.mockResolvedValueOnce({
      id: "1",
      connectionId: "conn-1",
      name: "My Query",
      sql: "SELECT 1",
      createdAt: "2024-01-01T00:00:00Z",
    });

    await service.save("conn-1", "My Query", "SELECT 1", "folder-1");

    expect(mockInvoke).toHaveBeenCalledWith("save_query", {
      connectionId: "conn-1",
      name: "My Query",
      sql: "SELECT 1",
      folder: "folder-1",
    });
  });

  it("save works without folder", async () => {
    mockInvoke.mockResolvedValueOnce({
      id: "1",
      connectionId: "conn-1",
      name: "My Query",
      sql: "SELECT 1",
      createdAt: "2024-01-01T00:00:00Z",
    });

    await service.save("conn-1", "My Query", "SELECT 1");

    expect(mockInvoke).toHaveBeenCalledWith("save_query", {
      connectionId: "conn-1",
      name: "My Query",
      sql: "SELECT 1",
      folder: undefined,
    });
  });

  it("listSaved calls list_saved_queries with connection_id", async () => {
    mockInvoke.mockResolvedValueOnce([]);

    await service.listSaved("conn-1");

    expect(mockInvoke).toHaveBeenCalledWith("list_saved_queries", {
      connectionId: "conn-1",
    });
  });

  it("deleteSaved calls delete_saved_query with id", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);

    await service.deleteSaved("query-1");

    expect(mockInvoke).toHaveBeenCalledWith("delete_saved_query", {
      id: "query-1",
    });
  });
});
