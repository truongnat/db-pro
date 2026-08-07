import { describe, expect, it, vi, beforeEach } from "vitest";

const { mockInvoke } = vi.hoisted(() => ({ mockInvoke: vi.fn() }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mockInvoke,
}));

import { QueryService, createQueryService } from "../services/query.service";

const service = new QueryService();

describe("QueryService — comprehensive", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
  });

  describe("cancel", () => {
    it("calls cancel_query with connectionId", async () => {
      mockInvoke.mockResolvedValueOnce(undefined);
      await service.cancel("conn-1");
      expect(mockInvoke).toHaveBeenCalledWith("cancel_query", {
        connectionId: "conn-1",
      });
    });
  });

  describe("executeMulti", () => {
    it("calls execute_query_multi with connectionId and sql", async () => {
      const multiResult = {
        results: [
          { columns: ["id"], rows: [[1]], rowCount: 1, durationMs: 5 },
          { columns: ["name"], rows: [["test"]], rowCount: 1, durationMs: 3 },
        ],
      };
      mockInvoke.mockResolvedValueOnce(multiResult);
      const result = await service.executeMulti("conn-1", "SELECT 1; SELECT 2;");
      expect(mockInvoke).toHaveBeenCalledWith("execute_query_multi", {
        connectionId: "conn-1",
        sql: "SELECT 1; SELECT 2;",
      });
      expect(result.results).toHaveLength(2);
    });
  });

  describe("save — run config operations", () => {
    it("saveRunConfig calls save_run_config with all params", async () => {
      const config = {
        id: "rc-1",
        connectionId: "conn-1",
        name: "Fast query",
        sql: "SELECT 1",
        timeoutMs: 5000,
        maxRows: 100,
      };
      mockInvoke.mockResolvedValueOnce(config);
      const result = await service.saveRunConfig("conn-1", "Fast query", "SELECT 1", 5000, 100);
      expect(mockInvoke).toHaveBeenCalledWith("save_run_config", {
        connectionId: "conn-1",
        name: "Fast query",
        sql: "SELECT 1",
        timeoutMs: 5000,
        maxRows: 100,
      });
      expect(result).toEqual(config);
    });

    it("listRunConfigs calls list_run_configs", async () => {
      mockInvoke.mockResolvedValueOnce([]);
      await service.listRunConfigs("conn-1");
      expect(mockInvoke).toHaveBeenCalledWith("list_run_configs", {
        connectionId: "conn-1",
      });
    });

    it("deleteRunConfig calls delete_run_config with id", async () => {
      mockInvoke.mockResolvedValueOnce(undefined);
      await service.deleteRunConfig("rc-1");
      expect(mockInvoke).toHaveBeenCalledWith("delete_run_config", {
        id: "rc-1",
      });
    });
  });

  describe("folder operations", () => {
    it("createFolder calls create_folder", async () => {
      const folder = { id: "f-1", connectionId: "conn-1", name: "My Folder" };
      mockInvoke.mockResolvedValueOnce(folder);
      const result = await service.createFolder("conn-1", "My Folder");
      expect(mockInvoke).toHaveBeenCalledWith("create_folder", {
        connectionId: "conn-1",
        name: "My Folder",
      });
      expect(result).toEqual(folder);
    });

    it("listFolders calls list_folders", async () => {
      mockInvoke.mockResolvedValueOnce([]);
      await service.listFolders("conn-1");
      expect(mockInvoke).toHaveBeenCalledWith("list_folders", {
        connectionId: "conn-1",
      });
    });

    it("deleteFolder calls delete_folder with id", async () => {
      mockInvoke.mockResolvedValueOnce(undefined);
      await service.deleteFolder("f-1");
      expect(mockInvoke).toHaveBeenCalledWith("delete_folder", {
        id: "f-1",
      });
    });
  });

  describe("createQueryService factory", () => {
    it("returns a QueryService instance", () => {
      const svc = createQueryService();
      expect(svc).toBeInstanceOf(QueryService);
    });
  });
});
