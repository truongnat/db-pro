import { describe, expect, it, vi } from "vitest";

const { mockInvoke } = vi.hoisted(() => ({ mockInvoke: vi.fn() }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mockInvoke,
}));

import { DataGridService, createDataGridService } from "../services/data-grid.service";

const service = new DataGridService();

describe("DataGridService", () => {
  it("fetchRows calls fetch_table_rows with snake_case args", async () => {
    mockInvoke.mockResolvedValueOnce({
      columns: [],
      rows: [],
      totalCount: 0,
      durationMs: 5,
    });

    await service.fetchRows("conn-1", {
      schema: "public",
      table: "users",
      filters: [{ column: "id", op: "eq", value: { type: "int64", value: "1" } }],
      sorts: [{ column: "name", direction: "asc" }],
      page: 2,
      pageSize: 25,
    });

    expect(mockInvoke).toHaveBeenCalledWith("fetch_table_rows", {
      connectionId: "conn-1",
      request: {
        schema: "public",
        table: "users",
        filters: [{ column: "id", op: "eq", value: { type: "int64", value: "1" } }],
        sorts: [{ column: "name", direction: "asc" }],
        page: 2,
        pageSize: 25,
      },
    });
  });

  it("insertRow calls insert_table_row", async () => {
    mockInvoke.mockResolvedValueOnce({ affectedRows: 1 });

    await service.insertRow("conn-1", {
      schema: "public",
      table: "users",
      columns: ["name"],
      values: [{ type: "text", value: "Alice" }],
    });

    expect(mockInvoke).toHaveBeenCalledWith("insert_table_row", {
      connectionId: "conn-1",
      request: {
        schema: "public",
        table: "users",
        columns: ["name"],
        values: [{ type: "text", value: "Alice" }],
      },
    });
  });

  it("updateRow calls update_table_row with pk info", async () => {
    mockInvoke.mockResolvedValueOnce({ affectedRows: 1 });

    await service.updateRow("conn-1", {
      schema: "public",
      table: "users",
      columns: ["name"],
      values: [{ type: "text", value: "Bob" }],
      pkColumns: ["id"],
      pkValues: [{ type: "int64", value: "1" }],
    });

    expect(mockInvoke).toHaveBeenCalledWith("update_table_row", {
      connectionId: "conn-1",
      request: {
        schema: "public",
        table: "users",
        columns: ["name"],
        values: [{ type: "text", value: "Bob" }],
        pkColumns: ["id"],
        pkValues: [{ type: "int64", value: "1" }],
      },
    });
  });

  it("deleteRow calls delete_table_row with pk info", async () => {
    mockInvoke.mockResolvedValueOnce({ affectedRows: 1 });

    await service.deleteRow("conn-1", {
      schema: "public",
      table: "users",
      columns: ["id", "name"],
      values: [
        { type: "int64", value: "1" },
        { type: "text", value: "Alice" },
      ],
      pkColumns: ["id"],
      pkValues: [{ type: "int64", value: "1" }],
    });

    expect(mockInvoke).toHaveBeenCalledWith("delete_table_row", {
      connectionId: "conn-1",
      request: {
        schema: "public",
        table: "users",
        columns: ["id", "name"],
        values: [
          { type: "int64", value: "1" },
          { type: "text", value: "Alice" },
        ],
        pkColumns: ["id"],
        pkValues: [{ type: "int64", value: "1" }],
      },
    });
  });

  it("createDataGridService returns a DataGridService", () => {
    const svc = createDataGridService();
    expect(svc).toBeInstanceOf(DataGridService);
  });
});
