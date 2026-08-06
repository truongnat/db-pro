import { apiInvoke } from "@/commons/utils/api";

import type { ExportResult } from "../types/export.types";

export class ExportService {
  async exportCsv(connectionId: string, sql: string): Promise<ExportResult> {
    return apiInvoke<ExportResult>("export_csv", {
      connection_id: connectionId,
      sql,
    });
  }

  async exportJson(connectionId: string, sql: string): Promise<ExportResult> {
    return apiInvoke<ExportResult>("export_json", {
      connection_id: connectionId,
      sql,
    });
  }

  async exportExcel(connectionId: string, sql: string): Promise<ExportResult> {
    return apiInvoke<ExportResult>("export_excel", {
      connection_id: connectionId,
      sql,
    });
  }
}

export function createExportService(): ExportService {
  return new ExportService();
}
