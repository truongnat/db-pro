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

  return (
    <div className="flex flex-col gap-1">
      <label className="text-sm font-medium text-foreground">
        {t("schema.ddlOperation")}
      </label>
      <select
        value={operation}
        onChange={(e) => onChange(e.target.value as DdlOperation)}
        className="h-9 rounded-sm border border-border bg-background px-3 py-2 text-sm text-foreground outline-none transition-colors focus:border-primary"
      >
        {OPERATIONS.map((op) => (
          <option key={op.value} value={op.value}>
            {t(op.labelKey)}
          </option>
        ))}
      </select>
    </div>
  );
}
