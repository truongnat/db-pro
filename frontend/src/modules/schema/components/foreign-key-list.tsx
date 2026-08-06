import { useTranslation } from "@/commons/locales/useTranslation";
import type { SchemaForeignKeyDto } from "../types/schema.types";
import { cn } from "@/lib/utils";

interface ForeignKeyListProps {
  foreignKeys: SchemaForeignKeyDto[];
}

export function ForeignKeyList({ foreignKeys }: ForeignKeyListProps) {
  const { t } = useTranslation();

  if (foreignKeys.length === 0) {
    return (
      <div className="p-4 text-sm text-muted-foreground">
        {t("schema.noForeignKeys")}
      </div>
    );
  }

  const headerClass = "px-3 py-2 font-medium text-muted-foreground border-b border-border";

  return (
    <table className="w-full text-sm">
      <thead>
        <tr>
          <th className={cn(headerClass, "text-left")}>{t("schema.fkName")}</th>
          <th className={cn(headerClass, "text-left")}>{t("schema.fkFromColumn")}</th>
          <th className={cn(headerClass, "text-left")}>{t("schema.fkToTable")}</th>
          <th className={cn(headerClass, "text-left")}>{t("schema.fkToColumn")}</th>
        </tr>
      </thead>
      <tbody>
        {foreignKeys.map((fk) => (
          <tr
            key={fk.name}
            className="transition-colors hover:bg-background"
          >
            <td className="px-3 py-1.5 font-mono text-sm">{fk.name}</td>
            <td className="px-3 py-1.5 font-mono text-xs text-muted-foreground">
              {fk.fromColumn}
            </td>
            <td className="px-3 py-1.5 font-mono text-xs">
              {fk.toSchema !== fk.schema ? `${fk.toSchema}.` : ""}{fk.toTable}
            </td>
            <td className="px-3 py-1.5 font-mono text-xs text-muted-foreground">
              {fk.toColumn}
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
