import { useState, useCallback, useEffect, useMemo } from "react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Checkbox } from "@/components/ui/checkbox";
import { Badge } from "@/components/ui/badge";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogCancel,
  AlertDialogAction,
} from "@/components/ui/alert-dialog";
import { useTranslation } from "@/commons/locales/useTranslation";
import { useExecuteDdl } from "@/modules/schema/queries/schema.queries";
import { AlertTriangle, Info, ShieldAlert, ShieldX } from "lucide-react";

import type { SchemaColumnDto } from "../types/schema.types";
import {
  classifyColumnMutation,
  hasChanges,
  type ColumnMutationDraft,
  type ClassifiedMutation,
  type MutationRiskLevel,
} from "../utils/column-mutation-risk";

interface ColumnEditDialogProps {
  column: SchemaColumnDto;
  schemaName: string;
  tableName: string;
  connectionId: string;
  onClose: () => void;
  onApplied: () => void;
}

const RISK_BADGE_VARIANT: Record<MutationRiskLevel, "success" | "info" | "warning" | "destructive"> = {
  low: "success",
  medium: "info",
  high: "warning",
  destructive: "destructive",
};

const RISK_ICON: Record<MutationRiskLevel, typeof Info> = {
  low: Info,
  medium: Info,
  high: ShieldAlert,
  destructive: ShieldX,
};

export function ColumnEditDialog({
  column,
  schemaName,
  tableName,
  connectionId,
  onClose,
  onApplied,
}: ColumnEditDialogProps) {
  const { t } = useTranslation();
  const executeDdl = useExecuteDdl(connectionId);

  // Draft state initialised from the current column
  const [newName, setNewName] = useState(column.name);
  const [newDataType, setNewDataType] = useState(column.dataType);
  const [newNullable, setNewNullable] = useState(column.nullable);
  const [newDefault, setNewDefault] = useState(column.defaultValue ?? "");
  const [showConfirm, setShowConfirm] = useState(false);
  const [applyError, setApplyError] = useState<string | null>(null);

  const draft: ColumnMutationDraft = {
    original: {
      name: column.name,
      dataType: column.dataType,
      nullable: column.nullable,
      defaultValue: column.defaultValue,
    },
    newName,
    newDataType,
    newNullable,
    newDefaultValue: newDefault || null,
  };

  const classified = useMemo(
    () => classifyColumnMutation(draft, schemaName, tableName),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [newName, newDataType, newNullable, newDefault, schemaName, tableName, column],
  );
  const changed = useMemo(() => hasChanges(draft), [draft]);

  // Cmd/Ctrl+Enter to apply (but opens confirmation if risky)
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "Enter" && changed) {
        e.preventDefault();
        handleApply();
      }
    },
    [changed, classified],
  );

  const handleApply = useCallback(() => {
    setApplyError(null);
    if (!changed || classified.sql.length === 0) return;

    if (classified.risk.requiresConfirmation) {
      setShowConfirm(true);
    } else {
      executeStatements();
    }
  }, [changed, classified]);

  const executeStatements = useCallback(async () => {
    setShowConfirm(false);
    setApplyError(null);

    try {
      for (const stmt of classified.sql) {
        await executeDdl.mutateAsync(stmt);
      }
      onApplied();
      onClose();
    } catch (err) {
      setApplyError(err instanceof Error ? err.message : String(err));
    }
  }, [classified.sql, executeDdl, onApplied, onClose]);

  // Close confirmation without applying
  const handleConfirmCancel = useCallback(() => {
    setShowConfirm(false);
  }, []);

  // Focus trap: auto-focus the name input on mount
  useEffect(() => {
    const timer = setTimeout(() => {
      const nameInput = document.getElementById("col-edit-name");
      nameInput?.focus();
    }, 100);
    return () => clearTimeout(timer);
  }, []);

  const RiskIcon = RISK_ICON[classified.risk.level];

  return (
    <>
      <Dialog open onOpenChange={(open) => { if (!open) onClose(); }}>
        <DialogContent className="sm:max-w-lg" onKeyDown={handleKeyDown}>
          <DialogHeader>
            <DialogTitle>Edit Column</DialogTitle>
            <DialogDescription>
              {schemaName}.{tableName}.{column.name}
            </DialogDescription>
          </DialogHeader>

          <div className="flex flex-col gap-4">
            {/* Rename */}
            <div>
              <Label htmlFor="col-edit-name" className="mb-1 block text-xs text-[var(--app-text-muted)]">
                Column name
              </Label>
              <Input
                id="col-edit-name"
                value={newName}
                onChange={(e) => setNewName(e.target.value)}
                className="w-full font-mono text-sm"
                placeholder={column.name}
              />
            </div>

            {/* Type */}
            <div>
              <Label htmlFor="col-edit-type" className="mb-1 block text-xs text-[var(--app-text-muted)]">
                Data type
              </Label>
              <Input
                id="col-edit-type"
                value={newDataType}
                onChange={(e) => setNewDataType(e.target.value)}
                className="w-full font-mono text-sm"
                placeholder={column.dataType}
              />
            </div>

            {/* Nullable */}
            <div className="flex items-center gap-2">
              <Checkbox
                id="col-edit-nullable"
                checked={newNullable}
                onCheckedChange={(checked) => setNewNullable(checked === true)}
              />
              <Label htmlFor="col-edit-nullable" className="text-sm font-normal">
                Nullable
              </Label>
            </div>

            {/* Default */}
            <div>
              <Label htmlFor="col-edit-default" className="mb-1 block text-xs text-[var(--app-text-muted)]">
                Default value
              </Label>
              <Input
                id="col-edit-default"
                value={newDefault}
                onChange={(e) => setNewDefault(e.target.value)}
                className="w-full font-mono text-sm"
                placeholder={column.defaultValue ?? "(none)"}
              />
              <p className="mt-1 text-[11px] text-[var(--app-text-muted)]">
                Leave empty to remove default. Use SQL expressions like <code className="rounded bg-muted px-1">now()</code>, <code className="rounded bg-muted px-1">0</code>, <code className="rounded bg-muted px-1">'value'</code>.
              </p>
            </div>

            {/* SQL Preview + Risk */}
            {changed && classified.operations.length > 0 && (
              <div className="rounded-md border border-[var(--app-border-subtle)] p-3">
                <div className="mb-2 flex items-center gap-2">
                  <RiskIcon className="h-3.5 w-3.5" />
                  <Badge variant={RISK_BADGE_VARIANT[classified.risk.level]} dot>
                    {classified.risk.label}
                  </Badge>
                </div>

                {/* Operations list */}
                <ul className="mb-2 space-y-0.5 text-xs text-[var(--app-text-muted)]">
                  {classified.operations.map((op, i) => (
                    <li key={i} className="flex items-start gap-1.5">
                      <span className="mt-0.5 text-[10px]">•</span>
                      <span>{op}</span>
                    </li>
                  ))}
                </ul>

                {/* SQL preview */}
                <pre className="overflow-x-auto rounded bg-muted p-2 font-mono text-[11px] leading-relaxed text-foreground">
                  {classified.sql.join("\n")}
                </pre>

                {/* Warnings */}
                {classified.warnings.length > 0 && (
                  <div className="mt-2 space-y-1">
                    {classified.warnings.map((w, i) => (
                      <div key={i} className="flex items-start gap-1.5 text-xs text-warning">
                        <AlertTriangle className="mt-0.5 h-3 w-3 shrink-0" />
                        <span>{w}</span>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            )}

            {/* Error */}
            {applyError && (
              <div className="rounded-sm bg-destructive/10 px-3 py-2 text-xs text-destructive">
                {applyError}
              </div>
            )}
          </div>

          <DialogFooter>
            <Button type="button" variant="outline" onClick={onClose}>
              {t("common.actions.cancel")}
            </Button>
            <Button
              type="button"
              disabled={!changed || executeDdl.isPending}
              onClick={handleApply}
            >
              {executeDdl.isPending
                ? t("common.states.loading")
                : classified.risk.requiresConfirmation
                  ? "Review & Apply…"
                  : "Apply"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Confirmation dialog for risky operations */}
      <AlertDialog open={showConfirm} onOpenChange={(open) => { if (!open) handleConfirmCancel(); }}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle className="flex items-center gap-2">
              <AlertTriangle className="h-4 w-4 text-warning" />
              Confirm Schema Change
            </AlertDialogTitle>
            <AlertDialogDescription>
              <span className="mb-2 block">
                <Badge variant={RISK_BADGE_VARIANT[classified.risk.level]} dot>
                  {classified.risk.label}
                </Badge>
              </span>
              {classified.risk.warning && (
                <span className="mb-2 block">{classified.risk.warning}</span>
              )}
              {classified.warnings.length > 0 && (
                <ul className="mt-2 space-y-1">
                  {classified.warnings.map((w, i) => (
                    <li key={i} className="flex items-start gap-1.5">
                      <AlertTriangle className="mt-0.5 h-3 w-3 shrink-0 text-warning" />
                      <span>{w}</span>
                    </li>
                  ))}
                </ul>
              )}
              <pre className="mt-3 overflow-x-auto rounded bg-muted p-2 font-mono text-[11px] leading-relaxed">
                {classified.sql.join("\n")}
              </pre>
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel onClick={handleConfirmCancel}>
              {t("common.actions.cancel")}
            </AlertDialogCancel>
            <AlertDialogAction onClick={executeStatements}>
              Apply Changes
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}
