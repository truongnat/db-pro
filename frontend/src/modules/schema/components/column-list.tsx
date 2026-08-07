import { useTranslation } from "@/commons/locales/useTranslation";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
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
      <div className="p-4 text-sm text-[var(--app-text-muted)]">
        {t("schema.noColumns")}
      </div>
    );
  }

  const sorted = sortColumnsForDisplay(columns);

  const headerClass = "px-3 py-2 font-medium text-[var(--app-text-muted)] border-b border-[var(--app-border-subtle)]";

  return (
    <Table className="w-full text-sm">
      <TableHeader>
        <TableRow>
          <TableHead className={cn(headerClass, "text-left")}>{t("schema.columnName")}</TableHead>
          <TableHead className={cn(headerClass, "text-left")}>{t("schema.columnDataType")}</TableHead>
          <TableHead className={cn(headerClass, "text-left")}>{t("schema.columnNullable")}</TableHead>
          <TableHead className={cn(headerClass, "text-left")}>{t("schema.columnDefault")}</TableHead>
          <TableHead className={cn(headerClass, "text-center")}>{t("schema.columnPk")}</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {sorted.map((col) => (
          <TableRow
            key={col.name}
            className="transition-colors hover:bg-background"
          >
            <TableCell className="px-3 py-1.5 font-mono text-sm">{col.name}</TableCell>
            <TableCell className="px-3 py-1.5 font-mono text-xs text-[var(--app-text-muted)]">
              {col.dataType}
            </TableCell>
            <TableCell className="px-3 py-1.5 text-[var(--app-text-muted)]">
              {col.nullable ? "YES" : "NO"}
            </TableCell>
            <TableCell className="px-3 py-1.5 font-mono text-xs text-[var(--app-text-muted)]">
              {col.defaultValue ?? "\u2014"}
            </TableCell>
            <TableCell className="px-3 py-1.5 text-center">
              {col.isPrimaryKey ? "\uD83D\uDD11" : ""}
            </TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  );
}
