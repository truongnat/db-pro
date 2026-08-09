import { useCallback, useMemo, useRef, useState } from "react";

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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";

import { useTranslation } from "@/commons/locales/useTranslation";

import { buildImportPreview, detectFormat, parseAllRows } from "../services/import-parser";
import type {
  ImportColumnMapping,
  ImportFormat,
  ImportPreview,
  ImportResult,
} from "../types/import.types";

interface ImportDialogProps {
  open: boolean;
  onClose: () => void;
  /** Target table name */
  tableName: string;
  /** Available columns in the target table */
  targetColumns: string[];
  /** Callback to execute a SQL statement */
  onExecuteSql: (sql: string) => void;
}

type ImportStep = "file" | "mapping" | "result";

function sqlValue(v: string): string {
  if (v === "") return "NULL";
  return `'${v.replace(/'/g, "''")}'`;
}

export function ImportDialog({
  open,
  onClose,
  tableName,
  targetColumns,
  onExecuteSql,
}: ImportDialogProps) {
  const { t } = useTranslation();
  const fileRef = useRef<HTMLInputElement>(null);
  const [step, setStep] = useState<ImportStep>("file");
  const [content, setContent] = useState("");
  const [format, setFormat] = useState<ImportFormat>("csv");
  const [preview, setPreview] = useState<ImportPreview | null>(null);
  const [mappings, setMappings] = useState<ImportColumnMapping[]>([]);
  const [result, setResult] = useState<ImportResult | null>(null);
  const [isImporting, setIsImporting] = useState(false);

  const reset = useCallback(() => {
    setStep("file");
    setContent("");
    setFormat("csv");
    setPreview(null);
    setMappings([]);
    setResult(null);
    setIsImporting(false);
  }, []);

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = (ev) => {
      const text = ev.target?.result;
      if (typeof text !== "string") return;
      setContent(text);
      const detected = detectFormat(text);
      setFormat(detected);
      const p = buildImportPreview(text, detected, targetColumns);
      setPreview(p);
      setMappings(p.mappings);
      setStep("mapping");
    };
    reader.readAsText(file);
    e.target.value = "";
  };

  const updateMapping = (index: number, targetColumn: string | null) => {
    setMappings((prev) => prev.map((m, i) => (i === index ? { ...m, targetColumn } : m)));
  };

  const mappedColumns = useMemo(() => mappings.filter((m) => m.targetColumn != null), [mappings]);

  const handleImport = async () => {
    if (!preview || mappedColumns.length === 0) return;
    setIsImporting(true);

    const allRows = parseAllRows(content, format);
    const colList = mappedColumns.map((m) => `"${m.targetColumn}"`).join(", ");
    let successCount = 0;
    const errors: ImportResult["errors"] = [];

    for (let i = 0; i < allRows.length; i++) {
      const row = allRows[i];
      const values = mappedColumns.map((m) => sqlValue(row[m.sourceName] ?? "")).join(", ");
      const sql = `INSERT INTO "${tableName}" (${colList}) VALUES (${values})`;
      try {
        onExecuteSql(sql);
        successCount++;
      } catch {
        errors.push({ row: i + 1, column: "-", message: "INSERT failed" });
      }
    }

    setResult({
      successCount,
      errorCount: errors.length,
      errors,
    });
    setIsImporting(false);
    setStep("result");
  };

  const handleClose = () => {
    reset();
    onClose();
  };

  return (
    <Dialog open={open} onOpenChange={(o) => !o && handleClose()}>
      <DialogContent className="sm:max-w-[560px]">
        <DialogHeader>
          <DialogTitle>
            {step === "file" && t("import.selectFile")}
            {step === "mapping" && t("import.mapColumns")}
            {step === "result" && t("import.importResult")}
          </DialogTitle>
        </DialogHeader>

        {/* Step 1: File selection */}
        {step === "file" && (
          <div className="space-y-3">
            <div className="space-y-1.5">
              <Label>{t("import.format")}</Label>
              <div className="flex gap-2">
                <Button
                  variant="outline"
                  className={
                    format === "csv"
                      ? "border-primary text-primary"
                      : "border-[var(--app-border-subtle)]"
                  }
                  onClick={() => setFormat("csv")}
                  type="button"
                >
                  CSV
                </Button>
                <Button
                  variant="outline"
                  className={
                    format === "json"
                      ? "border-primary text-primary"
                      : "border-[var(--app-border-subtle)]"
                  }
                  onClick={() => setFormat("json")}
                  type="button"
                >
                  JSON
                </Button>
              </div>
            </div>
            <div className="space-y-1.5">
              <Label>{t("import.file")}</Label>
              <Input
                ref={fileRef}
                type="file"
                accept=".csv,.json,text/csv,application/json"
                onChange={handleFileChange}
                className="h-8"
              />
            </div>
            <p className="text-xs text-[var(--app-text-muted)]">{t("import.hint")}</p>
          </div>
        )}

        {/* Step 2: Column mapping */}
        {step === "mapping" && preview && (
          <div className="space-y-3">
            <p className="text-xs text-[var(--app-text-muted)]">
              {t("import.rowsDetected", { count: preview.totalRowCount })}
            </p>

            {/* Mapping table */}
            <div className="max-h-40 space-y-1 overflow-auto rounded border border-[var(--app-border)]">
              {mappings.map((m, i) => (
                <div key={i} className="flex items-center gap-2 px-2 py-1">
                  <span className="w-24 truncate text-xs text-[var(--app-text-muted)]">
                    {m.sourceName}
                  </span>
                  <span className="text-xs">→</span>
                  <Select
                    value={m.targetColumn ?? "__none__"}
                    onValueChange={(v) => updateMapping(i, v === "__none__" ? null : v)}
                  >
                    <SelectTrigger className="h-7 flex-1 text-xs">
                      <SelectValue placeholder={t("import.skipColumn")} />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="__none__">{t("import.skipColumn")}</SelectItem>
                      {targetColumns.map((col) => (
                        <SelectItem key={col} value={col}>
                          {col}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
              ))}
            </div>

            {/* Preview table */}
            {preview.sampleRows.length > 0 && (
              <div className="max-h-32 overflow-auto rounded border border-[var(--app-border)]">
                <Table>
                  <TableHeader>
                    <TableRow>
                      {preview.sourceColumns.map((col) => (
                        <TableHead key={col} className="px-2 py-1 text-xs">
                          {col}
                        </TableHead>
                      ))}
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {preview.sampleRows.map((row, i) => (
                      <TableRow key={i}>
                        {preview.sourceColumns.map((col) => (
                          <TableCell key={col} className="px-2 py-0.5 text-xs">
                            {row[col]}
                          </TableCell>
                        ))}
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </div>
            )}

            <p className="text-xs text-[var(--app-text-muted)]">
              {t("import.mappedCount", {
                mapped: mappedColumns.length,
                total: mappings.length,
              })}
            </p>
          </div>
        )}

        {/* Step 3: Result */}
        {step === "result" && result && (
          <div className="space-y-3">
            <div className="rounded border border-[var(--app-border)] p-3">
              <p className="text-sm font-medium">{t("import.importComplete")}</p>
              <p className="text-xs text-[var(--app-text-muted)]">
                {t("import.successCount", { count: result.successCount })}
                {result.errorCount > 0 && (
                  <> · {t("import.errorCount", { count: result.errorCount })}</>
                )}
              </p>
            </div>

            {result.errors.length > 0 && (
              <div className="max-h-32 overflow-auto rounded border border-destructive/30">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead className="px-2 py-1 text-xs">#</TableHead>
                      <TableHead className="px-2 py-1 text-xs">{t("import.errorColumn")}</TableHead>
                      <TableHead className="px-2 py-1 text-xs">
                        {t("import.errorMessage")}
                      </TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {result.errors.slice(0, 50).map((err, i) => (
                      <TableRow key={i}>
                        <TableCell className="px-2 py-0.5 text-xs">{err.row}</TableCell>
                        <TableCell className="px-2 py-0.5 text-xs">{err.column}</TableCell>
                        <TableCell className="px-2 py-0.5 text-xs text-destructive">
                          {err.message}
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </div>
            )}
          </div>
        )}

        <DialogFooter>
          {step === "file" && (
            <Button type="button" variant="outline" onClick={handleClose}>
              {t("common.actions.cancel")}
            </Button>
          )}
          {step === "mapping" && (
            <>
              <Button type="button" variant="outline" onClick={() => setStep("file")}>
                {t("import.backToFile")}
              </Button>
              <Button
                type="button"
                onClick={handleImport}
                disabled={mappedColumns.length === 0 || isImporting}
              >
                {isImporting ? t("import.importing") : t("import.startImport")}
              </Button>
            </>
          )}
          {step === "result" && (
            <Button type="button" onClick={handleClose}>
              {t("common.actions.close")}
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
