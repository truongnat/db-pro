import { describe, expect, it, vi } from "vitest";

const { mockApiInvoke } = vi.hoisted(() => ({ mockApiInvoke: vi.fn() }));

vi.mock("@/commons/utils/api", () => ({
  apiInvoke: mockApiInvoke,
}));

import { ExportService, createExportService } from "../services/export.service";

describe("ExportService", () => {
  describe("exportCsv", () => {
    it("calls apiInvoke with export_csv command", async () => {
      const result = { fileContent: "a,b\n1,2", fileName: "export.csv", mimeType: "text/csv", rowCount: 1 };
      mockApiInvoke.mockResolvedValueOnce(result);

      const svc = new ExportService();
      const res = await svc.exportCsv("conn-1", "SELECT 1");

      expect(res).toEqual(result);
      expect(mockApiInvoke).toHaveBeenCalledWith("export_csv", {
        connectionId: "conn-1",
        sql: "SELECT 1",
      });
    });
  });

  describe("exportJson", () => {
    it("calls apiInvoke with export_json command", async () => {
      const result = { fileContent: "[]", fileName: "export.json", mimeType: "application/json", rowCount: 0 };
      mockApiInvoke.mockResolvedValueOnce(result);

      const svc = new ExportService();
      const res = await svc.exportJson("conn-1", "SELECT * FROM users");

      expect(res).toEqual(result);
      expect(mockApiInvoke).toHaveBeenCalledWith("export_json", {
        connectionId: "conn-1",
        sql: "SELECT * FROM users",
      });
    });
  });

  describe("exportExcel", () => {
    it("calls apiInvoke with export_excel command", async () => {
      const result = { fileContent: "base64...", fileName: "export.xlsx", mimeType: "application/vnd.ms-excel", rowCount: 5 };
      mockApiInvoke.mockResolvedValueOnce(result);

      const svc = new ExportService();
      const res = await svc.exportExcel("conn-1", "SELECT * FROM orders");

      expect(res).toEqual(result);
      expect(mockApiInvoke).toHaveBeenCalledWith("export_excel", {
        connectionId: "conn-1",
        sql: "SELECT * FROM orders",
      });
    });
  });

  describe("createExportService", () => {
    it("returns an ExportService instance", () => {
      const svc = createExportService();
      expect(svc).toBeInstanceOf(ExportService);
    });
  });
});
