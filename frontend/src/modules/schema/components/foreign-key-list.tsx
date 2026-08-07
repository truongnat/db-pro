import { useTranslation } from "@/commons/locales/useTranslation";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
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

  const headerClass = "px-3 py-2 font-medium text-[var(--app-text-muted)] border-b border-[var(--app-border-subtle)]";

  return (
    <Table className="w-full text-sm">
      <TableHeader>
        <TableRow>
          <TableHead className={cn(headerClass, "text-left")}>{t("schema.fkName")}</TableHead>
          <TableHead className={cn(headerClass, "text-left")}>{t("schema.fkFromColumn")}</TableHead>
          <TableHead className={cn(headerClass, "text-left")}>{t("schema.fkToTable")}</TableHead>
          <TableHead className={cn(headerClass, "text-left")}>{t("schema.fkToColumn")}</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {foreignKeys.map((fk) => (
          <TableRow
            key={fk.name}
            className="transition-colors hover:bg-background"
          >
            <TableCell className="px-3 py-1.5 font-mono text-sm">{fk.name}</TableCell>
            <TableCell className="px-3 py-1.5 font-mono text-xs text-muted-foreground">
              {fk.fromColumn}
            </TableCell>
            <TableCell className="px-3 py-1.5 font-mono text-xs">
              {fk.toSchema !== fk.schema ? `${fk.toSchema}.` : ""}{fk.toTable}
            </TableCell>
            <TableCell className="px-3 py-1.5 font-mono text-xs text-muted-foreground">
              {fk.toColumn}
            </TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  );
}
