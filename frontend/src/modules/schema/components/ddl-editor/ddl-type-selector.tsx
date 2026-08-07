import { useId } from "react";

import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { useTranslation } from "@/commons/locales/useTranslation";

import type { DdlOperation } from "../../services/ddl-builder";

interface DdlTypeSelectorProps {
  operation: DdlOperation;
  onChange: (op: DdlOperation) => void;
}

const OPERATIONS: { value: DdlOperation; labelKey: string }[] = [
  { value: "createTable", labelKey: "schema.ddlOp.createTable" },
  { value: "addColumn", labelKey: "schema.ddlOp.addColumn" },
  { value: "dropColumn", labelKey: "schema.ddlOp.dropColumn" },
  { value: "renameTable", labelKey: "schema.ddlOp.renameTable" },
  { value: "dropTable", labelKey: "schema.ddlOp.dropTable" },
  { value: "createView", labelKey: "schema.ddlOp.createView" },
  { value: "dropView", labelKey: "schema.ddlOp.dropView" },
  { value: "createIndex", labelKey: "schema.ddlOp.createIndex" },
  { value: "dropIndex", labelKey: "schema.ddlOp.dropIndex" },
];

export function DdlTypeSelector({ operation, onChange }: DdlTypeSelectorProps) {
  const { t } = useTranslation();
  const id = useId();

  return (
    <div className="flex flex-col gap-1">
      <Label htmlFor={id} className="text-sm font-medium text-foreground">
        {t("schema.ddlOperation")}
      </Label>
      <Select value={operation} onValueChange={(val) => onChange(val as DdlOperation)}>
        <SelectTrigger id={id} className="h-9 rounded-sm border border-[var(--app-border)] bg-background px-3 py-2 text-sm text-foreground outline-none transition-colors focus:border-primary">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {OPERATIONS.map((op) => (
            <SelectItem key={op.value} value={op.value}>
              {t(op.labelKey)}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );
}
