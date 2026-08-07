import { useState } from "react";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

import { useTranslation } from "@/commons/locales/useTranslation";

import { useExport } from "../queries/export.queries";
import { generateCsv, generateJson, generateSqlInserts } from "../services/export-generators";
import type { ColumnMeta, Row } from "@/modules/query/types/query.types";
import type { ExportFormat, ExportOptions } from "../types/export.types";
import { DEFAULT_EXPORT_OPTIONS } from "../types/export.types";
import { ExportProgress } from "./export-progress";

import { cn } from "@/lib/utils";

interface ExportDialogProps {
  open: boolean;
  onClose: () => void;
  connectionId: string | null;
  sql: string;
  columns: ColumnMeta[];
  rows: Row[];
  selectedRows?: Row[];
}

const FORMATS: { value: ExportFormat; labelKey: string }[] = [
  { value: "csv", labelKey: "export.csv" },
  { value: "json", labelKey: "export.json" },
  { value: "sql", labelKey: "export.sql" },
  { value: "excel", labelKey: "export.excel" },
];

const DELIMITERS: { value: ExportOptions["delimiter"]; label: string }[] = [
  { value: ",", label: "Comma (,)" },
  { value: ";", label: "Semicolon (;)" },
  { value: "\t", label: "Tab" },
  { value: "|", label: "Pipe (|)" },
];

function timestamp(): string {
  return new Date().toISOString().replace(/[:.]/g, "-").slice(0, 19);
}

function downloadText(content: string, fileName: string, mimeType: string) {
  const blob = new Blob([content], { type: mimeType });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = fileName;
  a.click();
  URL.revokeObjectURL(url);
}

export function ExportDialog({
  open,
  onClose,
  connectionId,
  sql,
  columns,
  rows,
  selectedRows,
}: ExportDialogProps) {
  const { t } = useTranslation();
  const [opts, setOpts] = useState<ExportOptions>({ ...DEFAULT_EXPORT_OPTIONS });

  const exportMutation = useExport(connectionId, "excel", sql);
  const hasSelected = (selectedRows?.length ?? 0) > 0;

  const update = <K extends keyof ExportOptions>(key: K, value: ExportOptions[K]) =>
    setOpts((prev) => ({ ...prev, [key]: value }));

  const handleExport = () => {
    const dataRows = opts.scope === "selected" && hasSelected ? selectedRows! : rows;
    const ts = timestamp();

    // Frontend-side generation for CSV, JSON, SQL
    if (opts.format === "csv") {
      const content = generateCsv(columns, dataRows, {
        delimiter: opts.delimiter,
        includeHeaders: opts.includeHeaders,
        nullRepresentation: opts.nullRepresentation,
      });
      downloadText(content, `export_${ts}.csv`, "text/csv");
      onClose();
      return;
    }

    if (opts.format === "json") {
      const content = generateJson(columns, dataRows, {
        pretty: opts.prettyJson,
        nullRepresentation: opts.nullRepresentation,
      });
      downloadText(content, `export_${ts}.json`, "application/json");
      onClose();
      return;
    }

    if (opts.format === "sql") {
      const content = generateSqlInserts(columns, dataRows, {
        tableName: opts.tableName || "table_name",
        nullRepresentation: opts.nullRepresentation,
      });
      downloadText(content, `export_${ts}.sql`, "text/sql");
      onClose();
      return;
    }

    // Backend-side for Excel
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
      <DialogContent className="sm:max-w-[448px]">
        <DialogHeader>
          <DialogTitle>{t("export.title")}</DialogTitle>
        </DialogHeader>

        <div className="space-y-3">
          {/* Format selector */}
          <div className="space-y-1.5">
            <Label>{t("export.selectFormat")}</Label>
            <div className="flex gap-2">
              {FORMATS.map((f) => (
                <Button
                  key={f.value}
                  variant="outline"
                  className={cn(
                    "flex-1",
                    opts.format === f.value
                      ? "border-primary bg-primary text-white"
                      : "border-[var(--app-border-subtle)] bg-transparent text-foreground",
                  )}
                  onClick={() => update("format", f.value)}
                  type="button"
                >
                  {t(f.labelKey)}
                </Button>
              ))}
            </div>
          </div>

          {/* Scope */}
          {hasSelected && (
            <div className="flex items-center justify-between">
              <Label>{t("export.scope")}</Label>
              <div className="flex gap-2">
                <Button
                  variant="outline"
                  size="sm"
                  className={cn(
                    opts.scope === "all"
                      ? "border-primary text-primary"
                      : "border-[var(--app-border-subtle)] text-foreground",
                  )}
                  onClick={() => update("scope", "all")}
                  type="button"
                >
                  {t("export.allRows")} ({rows.length})
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  className={cn(
                    opts.scope === "selected"
                      ? "border-primary text-primary"
                      : "border-[var(--app-border-subtle)] text-foreground",
                  )}
                  onClick={() => update("scope", "selected")}
                  type="button"
                >
                  {t("export.selectedRows")} ({selectedRows!.length})
                </Button>
              </div>
            </div>
          )}

          {/* CSV-specific options */}
          {opts.format === "csv" && (
            <>
              <div className="flex items-center justify-between">
                <Label>{t("export.delimiter")}</Label>
                <Select
                  value={opts.delimiter}
                  onValueChange={(v) => update("delimiter", v as ExportOptions["delimiter"])}
                >
                  <SelectTrigger className="w-[160px]">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {DELIMITERS.map((d) => (
                      <SelectItem key={d.value} value={d.value}>
                        {d.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="flex items-center justify-between">
                <Label>{t("export.includeHeaders")}</Label>
                <Switch
                  checked={opts.includeHeaders}
                  onCheckedChange={(v) => update("includeHeaders", v)}
                />
              </div>
            </>
          )}

          {/* SQL-specific options */}
          {opts.format === "sql" && (
            <div className="space-y-1.5">
              <Label>{t("export.tableName")}</Label>
              <Input
                value={opts.tableName}
                onChange={(e) => update("tableName", e.target.value)}
                placeholder="table_name"
                className="h-8"
              />
            </div>
          )}

          {/* JSON-specific options */}
          {opts.format === "json" && (
            <div className="flex items-center justify-between">
              <Label>{t("export.prettyPrint")}</Label>
              <Switch
                checked={opts.prettyJson}
                onCheckedChange={(v) => update("prettyJson", v)}
              />
            </div>
          )}

          {/* NULL representation (shared) */}
          {(opts.format === "csv" || opts.format === "json" || opts.format === "sql") && (
            <div className="space-y-1.5">
              <Label>{t("export.nullRepresentation")}</Label>
              <Input
                value={opts.nullRepresentation}
                onChange={(e) => update("nullRepresentation", e.target.value)}
                placeholder={opts.format === "sql" ? "NULL" : ""}
                className="h-8"
              />
            </div>
          )}

          {/* SQL preview */}
          {opts.format !== "excel" && (
            <div className="space-y-1.5">
              <Label>SQL</Label>
              <pre className="max-h-16 overflow-auto rounded-sm border border-[var(--app-border-subtle)] bg-background p-2 text-xs text-foreground">
                {sql}
              </pre>
            </div>
          )}

          {exportMutation.isPending && (
            <ExportProgress format={opts.format} rowCount={null} />
          )}

          {exportMutation.isError && (
            <p className="text-xs text-destructive">{t("export.failed")}</p>
          )}
        </div>

        <DialogFooter>
          <Button type="button" variant="outline" onClick={onClose}>
            {t("common.actions.cancel")}
          </Button>
          <Button
            type="button"
            onClick={handleExport}
            disabled={exportMutation.isPending}
          >
            {t("export.title")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
