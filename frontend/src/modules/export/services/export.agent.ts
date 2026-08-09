import type { ExportResult } from "../types/export.types";

function base64Encode(text: string): string {
  return btoa(text);
}

export class MockExportService {
  async exportCsv(_connectionId: string, _sql: string): Promise<ExportResult> {
    const content = "id,name\n1,Alice\n2,Bob\n";
    return {
      fileContent: base64Encode(content),
      fileName: "export.csv",
      mimeType: "text/csv",
      rowCount: 2,
    };
  }

  async exportJson(_connectionId: string, _sql: string): Promise<ExportResult> {
    const content = JSON.stringify([
      { id: 1, name: "Alice" },
      { id: 2, name: "Bob" },
    ]);
    return {
      fileContent: base64Encode(content),
      fileName: "export.json",
      mimeType: "application/json",
      rowCount: 2,
    };
  }

  async exportExcel(_connectionId: string, _sql: string): Promise<ExportResult> {
    return {
      fileContent: base64Encode("mock-excel-content"),
      fileName: "export.xlsx",
      mimeType: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
      rowCount: 2,
    };
  }
}

export function createMockExportService(): MockExportService {
  return new MockExportService();
}
