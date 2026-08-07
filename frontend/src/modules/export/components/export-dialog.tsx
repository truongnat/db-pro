import { useState } from "react";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";

import { useTranslation } from "@/commons/locales/useTranslation";

import { cn } from "@/lib/utils";

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
    <Dialog open={open} onOpenChange={(o) => !o && onClose()}>
      <DialogContent className="sm:max-w-[384px]">
        <DialogHeader>
          <DialogTitle>{t("export.title")}</DialogTitle>
        </DialogHeader>

        <div className="space-y-3">
          <div className="space-y-1.5">
            <Label>{t("export.selectFormat")}</Label>
            <div className="flex gap-2">
              {FORMATS.map((f) => (
                <Button
                  key={f.value}
                  variant="outline"
                  className={cn(
                    "flex-1",
                    format === f.value
                      ? "border-primary bg-primary text-white"
                      : "border-border bg-transparent text-foreground",
                  )}
                  onClick={() => setFormat(f.value)}
                  type="button"
                >
                  {t(f.labelKey)}
                </Button>
              ))}
            </div>
          </div>

          <div className="space-y-1.5">
            <Label>SQL</Label>
            <pre className="max-h-24 overflow-auto rounded-sm border border-border bg-background p-2 text-xs text-foreground">
              {sql}
            </pre>
          </div>

          {exportMutation.isPending && (
            <ExportProgress format={format} rowCount={null} />
          )}

          {exportMutation.isError && (
            <p className="text-xs text-destructive">
              {t("export.failed")}
            </p>
          )}
        </div>

        <DialogFooter>
          <Button type="button" variant="outline" onClick={onClose}>
            {t("common.actions.cancel")}
          </Button>
          <Button type="button" onClick={handleExport} disabled={exportMutation.isPending}>
            {t("export.title")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
