import { useState } from "react";

import { useTranslation } from "@/commons/locales/useTranslation";

import { useExport } from "../queries/export.queries";
import type { ExportFormat } from "../types/export.types";
import { ExportProgress } from "./export-progress";

interface ExportDialogProps {
  open: boolean;
  onClose: () => void;
  connectionId: string | null;
  sql: string;
}

const FORMATS: { value: ExportFormat; labelKey: string }[] = [
  { value: "csv", labelKey: "export.csv" },
  { value: "json", labelKey: "export.json" },
  { value: "excel", labelKey: "export.excel" },
];

export function ExportDialog({ open, onClose, connectionId, sql }: ExportDialogProps) {
  const { t } = useTranslation();
  const [format, setFormat] = useState<ExportFormat>("csv");

  const exportMutation = useExport(connectionId, format, sql);

  if (!open) return null;

  const handleExport = () => {
    exportMutation.mutate(undefined, {
      onSuccess: (result) => {
        const binary = atob(result.fileContent);
        const bytes = new Uint8Array(binary.length);
        for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
        const blob = new Blob([bytes], { type: result.mimeType });
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.href = url;
        a.download = result.fileName;
        a.click();
        URL.revokeObjectURL(url);
        onClose();
      },
    });
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center" style={{ backgroundColor: "rgba(0,0,0,0.4)" }}>
      <div
        className="w-96 rounded-[var(--radius-md)] p-4 shadow-lg"
        style={{ backgroundColor: "var(--color-surface)", border: "1px solid var(--color-border)" }}
      >
        <h2 className="mb-3 text-sm font-semibold" style={{ color: "var(--color-text)" }}>
          {t("export.title")}
        </h2>

        <div className="mb-3">
          <label className="mb-1 block text-xs" style={{ color: "var(--color-text-secondary)" }}>
            {t("export.selectFormat")}
          </label>
          <div className="flex gap-2">
            {FORMATS.map((f) => (
              <button
                key={f.value}
                className="flex-1 rounded-[var(--radius-sm)] border px-3 py-1.5 text-xs transition-colors"
                style={{
                  borderColor: format === f.value ? "var(--color-primary, #3b82f6)" : "var(--color-border)",
                  backgroundColor: format === f.value ? "var(--color-primary, #3b82f6)" : "transparent",
                  color: format === f.value ? "white" : "var(--color-text)",
                }}
                onClick={() => setFormat(f.value)}
                type="button"
              >
                {t(f.labelKey)}
              </button>
            ))}
          </div>
        </div>

        <div className="mb-3">
          <label className="mb-1 block text-xs" style={{ color: "var(--color-text-secondary)" }}>
            SQL
          </label>
          <pre
            className="max-h-24 overflow-auto rounded-[var(--radius-sm)] border p-2 text-xs"
            style={{ borderColor: "var(--color-border)", color: "var(--color-text)", backgroundColor: "var(--color-bg)" }}
          >
            {sql}
          </pre>
        </div>

        {exportMutation.isPending && (
          <ExportProgress format={format} rowCount={null} />
        )}

        {exportMutation.isError && (
          <p className="mb-2 text-xs" style={{ color: "var(--color-error, #ef4444)" }}>
            {t("export.failed")}
          </p>
        )}

        <div className="flex justify-end gap-2">
          <button
            className="rounded-[var(--radius-sm)] border px-3 py-1.5 text-xs transition-colors hover:bg-[var(--color-bg)]"
            style={{ borderColor: "var(--color-border)", color: "var(--color-text)" }}
            onClick={onClose}
            type="button"
          >
            {t("common.actions.cancel")}
          </button>
          <button
            className="rounded-[var(--radius-sm)] px-3 py-1.5 text-xs font-medium text-white transition-colors disabled:opacity-50"
            style={{ backgroundColor: "var(--color-primary, #3b82f6)" }}
            onClick={handleExport}
            disabled={exportMutation.isPending}
            type="button"
          >
            {t("export.title")}
          </button>
        </div>
      </div>
    </div>
  );
}
