import { describe, expect, it, vi, beforeEach } from "vitest";

const { mockInvoke } = vi.hoisted(() => ({ mockInvoke: vi.fn() }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mockInvoke,
}));

import { SchemaService } from "../services/schema.service";

const service = new SchemaService();

describe("SchemaService — comprehensive", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
  });

  describe("executeDdl", () => {
    it("calls execute_ddl with connectionId and sql", async () => {
      mockInvoke.mockResolvedValueOnce({ affectedRows: 0 });
      const result = await service.executeDdl("conn-1", "CREATE TABLE t (id INT)");
      expect(mockInvoke).toHaveBeenCalledWith("execute_ddl", {
        connectionId: "conn-1",
        sql: "CREATE TABLE t (id INT)",
      });
      expect(result).toEqual({ affectedRows: 0 });
    });
  });

  describe("diffSchemas", () => {
    it("calls diff_schemas with source and target ids", async () => {
      const diff = {
        tablesOnlyInSource: ["t1"],
        tablesOnlyInTarget: ["t2"],
        columnDiffs: [],
        indexesOnlyInSource: [],
        indexesOnlyInTarget: ["idx_new"],
      };
      mockInvoke.mockResolvedValueOnce(diff);
      const result = await service.diffSchemas("src-conn", "tgt-conn");
      expect(mockInvoke).toHaveBeenCalledWith("diff_schemas", {
        sourceId: "src-conn",
        targetId: "tgt-conn",
      });
      expect(result).toEqual(diff);
    });
  });

  describe("diffTableData", () => {
    it("calls diff_table_data with all parameters", async () => {
      const dataDiff = {
        schema: "public",
        table: "users",
        sourceRowCount: 100,
        targetRowCount: 80,
        rowCountDiff: 20,
      };
      mockInvoke.mockResolvedValueOnce(dataDiff);
      const result = await service.diffTableData("src-conn", "tgt-conn", "public", "users");
      expect(mockInvoke).toHaveBeenCalledWith("diff_table_data", {
        sourceId: "src-conn",
        targetId: "tgt-conn",
        schema: "public",
        table: "users",
      });
      expect(result).toEqual(dataDiff);
    });
  });

  describe("getObjectDependencies", () => {
    it("calls get_object_dependencies with all parameters", async () => {
      const deps = [
        {
          objectType: "view",
          objectName: "v_users",
          dependsOnType: "table",
          dependsOnName: "users",
        },
      ];
      mockInvoke.mockResolvedValueOnce(deps);
      const result = await service.getObjectDependencies("conn-1", "public", "users");
      expect(mockInvoke).toHaveBeenCalledWith("get_object_dependencies", {
        connectionId: "conn-1",
        schema: "public",
        objectName: "users",
      });
      expect(result).toEqual(deps);
    });

    it("returns empty array when no dependencies", async () => {
      mockInvoke.mockResolvedValueOnce([]);
      const result = await service.getObjectDependencies("conn-1", "public", "logs");
      expect(result).toEqual([]);
    });
  });

  describe("listPartitions", () => {
    it("calls list_partitions with connectionId", async () => {
      const partitions = [
        { schema: "public", table: "events", partitionStrategy: "RANGE", partitions: [] },
      ];
      mockInvoke.mockResolvedValueOnce(partitions);
      const result = await service.listPartitions("conn-1");
      expect(mockInvoke).toHaveBeenCalledWith("list_partitions", {
        connectionId: "conn-1",
      });
      expect(result).toEqual(partitions);
    });
  });

  describe("listTablespaces", () => {
    it("calls list_tablespaces with connectionId", async () => {
      const tablespaces = [{ name: "pg_default", owner: "postgres", location: "" }];
      mockInvoke.mockResolvedValueOnce(tablespaces);
      const result = await service.listTablespaces("conn-1");
      expect(mockInvoke).toHaveBeenCalledWith("list_tablespaces", {
        connectionId: "conn-1",
      });
      expect(result).toEqual(tablespaces);
    });
  });

  describe("renameSchemaObject", () => {
    it("calls rename_schema_object with all parameters", async () => {
      mockInvoke.mockResolvedValueOnce(undefined);
      await service.renameSchemaObject("conn-1", "table", "public", "users", "accounts");
      expect(mockInvoke).toHaveBeenCalledWith("rename_schema_object", {
        connectionId: "conn-1",
        objectType: "table",
        schema: "public",
        oldName: "users",
        newName: "accounts",
      });
    });
  });

  describe("createSchemaService factory", () => {
    it("returns a SchemaService instance", async () => {
      const { createSchemaService } = await import("../services/schema.service");
      const svc = createSchemaService();
      expect(svc).toBeInstanceOf(SchemaService);
    });
  });
});
