import { describe, expect, it, vi } from "vitest";

const { mockInvoke } = vi.hoisted(() => ({ mockInvoke: vi.fn() }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mockInvoke,
}));

import { ExportService } from "../services/export.service";

const service = new ExportService();

const mockResult = {
  fileContent: "aWQsbmFtZQoxLEFsaWNl",
  fileName: "export.csv",
  mimeType: "text/csv",
  rowCount: 1,
};

describe("ExportService", () => {
  it("exportCsv calls export_csv with connection_id and sql", async () => {
    mockInvoke.mockResolvedValueOnce(mockResult);

    await service.exportCsv("conn-1", "SELECT * FROM users");

    expect(mockInvoke).toHaveBeenCalledWith("export_csv", {
      connection_id: "conn-1",
      sql: "SELECT * FROM users",
    });
  });

  it("exportJson calls export_json with connection_id and sql", async () => {
    mockInvoke.mockResolvedValueOnce({ ...mockResult, fileName: "export.json", mimeType: "application/json" });

    await service.exportJson("conn-1", "SELECT 1");

    expect(mockInvoke).toHaveBeenCalledWith("export_json", {
      connection_id: "conn-1",
      sql: "SELECT 1",
    });
  });

  it("exportExcel calls export_excel with connection_id and sql", async () => {
    mockInvoke.mockResolvedValueOnce({ ...mockResult, fileName: "export.xlsx" });

    await service.exportExcel("conn-1", "SELECT 1");

    expect(mockInvoke).toHaveBeenCalledWith("export_excel", {
      connection_id: "conn-1",
      sql: "SELECT 1",
    });
  });
});
