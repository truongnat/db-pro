import { useTranslation } from "@/commons/locales/useTranslation";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Key } from "lucide-react";
import type { SchemaColumnDto } from "../types/schema.types";
import { sortColumnsForDisplay } from "../types/schema.types";
import { cn } from "@/lib/utils";

interface ColumnListProps {
  columns: SchemaColumnDto[];
}

export function ColumnList({ columns }: ColumnListProps) {
  const { t } = useTranslation();

  if (columns.length === 0) {
    return <div className="p-4 text-sm text-[var(--app-text-muted)]">{t("schema.noColumns")}</div>;
  }

  const sorted = sortColumnsForDisplay(columns);

  const headerClass =
    "px-3 py-2 font-medium text-[12.5px] text-[var(--app-text-muted)] border-b border-[var(--app-border-subtle)]";

  return (
    <Table className="w-full text-[13px]">
      <TableHeader>
        <TableRow>
          <TableHead className={cn(headerClass, "text-left")}>{t("schema.columnName")}</TableHead>
          <TableHead className={cn(headerClass, "text-left")}>
            {t("schema.columnDataType")}
          </TableHead>
          <TableHead className={cn(headerClass, "text-left")}>
            {t("schema.columnNullable")}
          </TableHead>
          <TableHead className={cn(headerClass, "text-left")}>
            {t("schema.columnDefault")}
          </TableHead>
          <TableHead className={cn(headerClass, "text-center")}>{t("schema.columnPk")}</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {sorted.map((col) => (
          <TableRow key={col.name} className="transition-colors hover:bg-[var(--app-hover)]">
            <TableCell className="px-3 py-1.5 font-mono text-[13px]">{col.name}</TableCell>
            <TableCell className="px-3 py-1.5 font-mono text-[11px] text-[var(--app-text-muted)]">
              {col.dataType}
            </TableCell>
            <TableCell className="px-3 py-1.5 text-[12px] text-[var(--app-text-muted)]">
              {col.nullable ? "YES" : "NO"}
            </TableCell>
            <TableCell className="px-3 py-1.5 font-mono text-[11px] text-[var(--app-text-muted)]">
              {col.defaultValue ?? "\u2014"}
            </TableCell>
            <TableCell className="px-3 py-1.5 text-center">
              {col.isPrimaryKey && <Key className="inline h-3.5 w-3.5 text-primary" />}
            </TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  );
}
