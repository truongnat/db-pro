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
import { useExecuteDdlBatch } from "@/modules/schema/queries/schema.queries";
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

function DiffRow({ label, from, to }: { label: string; from: string; to: string }) {
  const changed = from !== to;
  return (
    <div className="flex items-baseline gap-2 text-xs">
      <span className="w-16 shrink-0 text-[var(--app-text-muted)]">{label}</span>
      <span className={`font-mono ${changed ? "line-through opacity-50" : ""}`}>{from || "—"}</span>
      {changed && (
        <>
          <span className="text-[var(--app-text-muted)]">→</span>
          <span className="font-mono font-medium">{to || "—"}</span>
        </>
      )}
    </div>
  );
}

export function ColumnEditDialog({
  column,
  schemaName,
  tableName,
  connectionId,
  onClose,
  onApplied,
}: ColumnEditDialogProps) {
  const { t } = useTranslation();
  const executeBatch = useExecuteDdlBatch(connectionId);

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

  const nameHasSpace = newName.includes(" ") && newName !== column.name;

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
      await executeBatch.mutateAsync(classified.sql);
      onApplied();
      onClose();
    } catch (err) {
      setApplyError(err instanceof Error ? err.message : String(err));
    }
  }, [classified.sql, executeBatch, onApplied, onClose]);

  const handleConfirmCancel = useCallback(() => {
    setShowConfirm(false);
  }, []);

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
        <DialogContent className="sm:max-w-md" onKeyDown={handleKeyDown}>
          <DialogHeader>
            <DialogTitle className="text-sm">Edit Column</DialogTitle>
            <DialogDescription className="font-mono text-xs">
              {schemaName}.{tableName}
            </DialogDescription>
          </DialogHeader>

          <div className="flex flex-col gap-3">
            {/* Diff summary */}
            <div className="rounded-md border border-[var(--app-border-subtle)] p-2.5">
              <DiffRow label="Name" from={column.name} to={newName} />
              <DiffRow label="Type" from={column.dataType} to={newDataType} />
              <DiffRow
                label="Nullable"
                from={column.nullable ? "YES" : "NO"}
                to={newNullable ? "YES" : "NO"}
              />
              <DiffRow
                label="Default"
                from={column.defaultValue ?? ""}
                to={newDefault}
              />
            </div>

            {/* Editable fields — compact */}
            <div className="grid grid-cols-2 gap-2">
              <div>
                <Label htmlFor="col-edit-name" className="mb-0.5 block text-[11px] text-[var(--app-text-muted)]">
                  Column name
                </Label>
                <Input
                  id="col-edit-name"
                  value={newName}
                  onChange={(e) => setNewName(e.target.value)}
                  className="h-7 w-full font-mono text-xs"
                  placeholder={column.name}
                />
                {nameHasSpace && (
                  <p className="mt-0.5 text-[10px] text-warning">
                    Contains spaces — will require quoting in SQL.
                  </p>
                )}
              </div>
              <div>
                <Label htmlFor="col-edit-type" className="mb-0.5 block text-[11px] text-[var(--app-text-muted)]">
                  Data type
                </Label>
                <Input
                  id="col-edit-type"
                  value={newDataType}
                  onChange={(e) => setNewDataType(e.target.value)}
                  className="h-7 w-full font-mono text-xs"
                  placeholder={column.dataType}
                />
              </div>
            </div>

            <div className="flex items-center gap-2">
              <Checkbox
                id="col-edit-nullable"
                checked={newNullable}
                onCheckedChange={(checked) => setNewNullable(checked === true)}
              />
              <Label htmlFor="col-edit-nullable" className="text-xs font-normal">
                Nullable
              </Label>
            </div>

            <div>
              <Label htmlFor="col-edit-default" className="mb-0.5 block text-[11px] text-[var(--app-text-muted)]">
                Default value
              </Label>
              <Input
                id="col-edit-default"
                value={newDefault}
                onChange={(e) => setNewDefault(e.target.value)}
                className="h-7 w-full font-mono text-xs"
                placeholder={column.defaultValue ?? "(none)"}
              />
            </div>

            {/* SQL Preview + Risk */}
            {changed && classified.operations.length > 0 && (
              <div className="rounded-md border border-[var(--app-border-subtle)] p-2.5">
                <div className="mb-1.5 flex items-center justify-between">
                  <div className="flex items-center gap-1.5">
                    <RiskIcon className="h-3 w-3" />
                    <Badge variant={RISK_BADGE_VARIANT[classified.risk.level]} dot>
                      {classified.risk.label}
                    </Badge>
                  </div>
                  <span className="text-[10px] text-[var(--app-text-muted)]">
                    {classified.operations.length} change{classified.operations.length > 1 ? "s" : ""}
                  </span>
                </div>

                <ul className="mb-1.5 space-y-0.5 text-[11px] text-[var(--app-text-muted)]">
                  {classified.operations.map((op, i) => (
                    <li key={i} className="flex items-start gap-1">
                      <span className="mt-px text-[8px]">•</span>
                      <span>{op}</span>
                    </li>
                  ))}
                </ul>

                <pre className="overflow-x-auto rounded bg-muted p-1.5 font-mono text-[10px] leading-relaxed text-foreground">
                  {classified.sql.join("\n")}
                </pre>

                {classified.warnings.length > 0 && (
                  <div className="mt-1.5 space-y-0.5">
                    {classified.warnings.map((w, i) => (
                      <div key={i} className="flex items-start gap-1 text-[10px] text-warning">
                        <AlertTriangle className="mt-px h-2.5 w-2.5 shrink-0" />
                        <span>{w}</span>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            )}

            {applyError && (
              <div className="rounded-sm bg-destructive/10 px-2.5 py-1.5 text-[11px] text-destructive">
                {applyError}
              </div>
            )}
          </div>

          <DialogFooter>
            <Button type="button" variant="outline" size="sm" onClick={onClose}>
              {t("common.actions.cancel")}
            </Button>
            <Button
              type="button"
              size="sm"
              disabled={!changed || executeBatch.isPending}
              onClick={handleApply}
            >
              {executeBatch.isPending
                ? t("common.states.loading")
                : classified.risk.requiresConfirmation
                  ? "Review & Apply…"
                  : "Apply"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <AlertDialog open={showConfirm} onOpenChange={(open) => { if (!open) handleConfirmCancel(); }}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle className="flex items-center gap-2 text-sm">
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
                <span className="mb-2 block text-xs">{classified.risk.warning}</span>
              )}
              {classified.warnings.length > 0 && (
                <ul className="mt-2 space-y-1">
                  {classified.warnings.map((w, i) => (
                    <li key={i} className="flex items-start gap-1.5 text-xs">
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
