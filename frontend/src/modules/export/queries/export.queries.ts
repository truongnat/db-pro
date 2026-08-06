import { useMutation } from "@tanstack/react-query";

import { container } from "@/app/app.module";
import { SERVICE_NAMES, type IExportService } from "@/commons/di/registry";

import type { ExportFormat, ExportResult } from "../types/export.types";

function getExportService() {
  return container.resolve<IExportService>(SERVICE_NAMES.EXPORT_SERVICE);
}

export function useExport(
  connectionId: string | null,
  format: ExportFormat,
  sql: string,
) {
  const methodMap = {
    csv: "exportCsv" as const,
    json: "exportJson" as const,
    excel: "exportExcel" as const,
  };

  return useMutation({
    mutationFn: () => {
      const service = getExportService();
      return service[methodMap[format]](connectionId!, sql) as Promise<ExportResult>;
    },
    onError: (err: unknown) => {
      console.error(`[Export] ${format} export failed:`, err);
    },
  });
}
