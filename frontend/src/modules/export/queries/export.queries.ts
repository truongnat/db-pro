import { useMutation } from "@tanstack/react-query";

import { container } from "@/app/app.module";
import { SERVICE_NAMES, type IExportService } from "@/commons/di/registry";

import type { ExportResult } from "../types/export.types";

function getExportService() {
  return container.resolve<IExportService>(SERVICE_NAMES.EXPORT_SERVICE);
}

type BackendExportFormat = "csv" | "json" | "excel";

export function useExport(connectionId: string | null, format: BackendExportFormat, sql: string) {
  const methodMap: Record<BackendExportFormat, keyof IExportService> = {
    csv: "exportCsv",
    json: "exportJson",
    excel: "exportExcel",
  };

  return useMutation({
    mutationFn: () => {
      const service = getExportService();
      const method = methodMap[format];
      return service[method](connectionId!, sql) as Promise<ExportResult>;
    },
    onError: (err: unknown) => {
      console.error(`[Export] ${format} export failed:`, err);
    },
  });
}
