import { useTranslation } from "@/commons/locales/useTranslation";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import type { SchemaIndexDto } from "../types/schema.types";
import { cn } from "@/lib/utils";

interface IndexListProps {
  indexes: SchemaIndexDto[];
}

export function IndexList({ indexes }: IndexListProps) {
  const { t } = useTranslation();

  if (indexes.length === 0) {
    return <div className="p-4 text-sm text-[var(--text-secondary)]">{t("schema.noIndexes")}</div>;
  }

  const headerClass =
    "px-3 py-2 font-medium text-[var(--text-secondary)] border-b border-[var(--border-subtle)]";

  return (
    <Table className="w-full text-sm">
      <TableHeader>
        <TableRow>
          <TableHead className={cn(headerClass, "text-left")}>{t("schema.indexName")}</TableHead>
          <TableHead className={cn(headerClass, "text-left")}>{t("schema.indexColumns")}</TableHead>
          <TableHead className={cn(headerClass, "text-center")}>
            {t("schema.indexUnique")}
          </TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {indexes.map((idx) => (
          <TableRow key={idx.name} className="transition-colors hover:bg-background">
            <TableCell className="px-3 py-1.5 font-mono text-sm">{idx.name}</TableCell>
            <TableCell className="px-3 py-1.5 font-mono text-xs text-[var(--text-secondary)]">
              {idx.columns.join(", ")}
            </TableCell>
            <TableCell className="px-3 py-1.5 text-center text-[var(--text-secondary)]">
              {idx.unique ? "YES" : "NO"}
            </TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  );
}
