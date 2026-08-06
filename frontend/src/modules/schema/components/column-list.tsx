import { useTranslation } from "@/commons/locales/useTranslation";
import type { SchemaColumnDto } from "../types/schema.types";
import { sortColumnsForDisplay } from "../types/schema.types";
import { cn } from "@/lib/utils";

interface ColumnListProps {
  columns: SchemaColumnDto[];
}

export function ColumnList({ columns }: ColumnListProps) {
  const { t } = useTranslation();

  if (columns.length === 0) {
    return (
      <div className="p-4 text-sm text-muted-foreground">
        {t("schema.noColumns")}
      </div>
    );
  }

  const sorted = sortColumnsForDisplay(columns);

  const headerClass = "px-3 py-2 font-medium text-muted-foreground border-b border-border";

  return (
    <table className="w-full text-sm">
      <thead>
        <tr>
          <th className={cn(headerClass, "text-left")}>{t("schema.columnName")}</th>
          <th className={cn(headerClass, "text-left")}>{t("schema.columnDataType")}</th>
          <th className={cn(headerClass, "text-left")}>{t("schema.columnNullable")}</th>
          <th className={cn(headerClass, "text-left")}>{t("schema.columnDefault")}</th>
          <th className={cn(headerClass, "text-center")}>{t("schema.columnPk")}</th>
        </tr>
      </thead>
      <tbody>
        {sorted.map((col) => (
          <tr
            key={col.name}
            className="transition-colors hover:bg-background"
          >
            <td className="px-3 py-1.5 font-mono text-sm">{col.name}</td>
            <td className="px-3 py-1.5 font-mono text-xs text-muted-foreground">
              {col.dataType}
            </td>
            <td className="px-3 py-1.5 text-muted-foreground">
              {col.nullable ? "YES" : "NO"}
            </td>
            <td className="px-3 py-1.5 font-mono text-xs text-muted-foreground">
              {col.defaultValue ?? "\u2014"}
            </td>
            <td className="px-3 py-1.5 text-center">
              {col.isPrimaryKey ? "\uD83D\uDD11" : ""}
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
