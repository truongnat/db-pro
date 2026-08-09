import { useState } from "react";
import { useTranslation } from "@/commons/locales/useTranslation";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { Key, Copy, Check, Pencil } from "lucide-react";
import type { SchemaColumnDto } from "../types/schema.types";
import { sortColumnsForDisplay } from "../types/schema.types";
import { cn } from "@/lib/utils";

interface ColumnListProps {
  columns: SchemaColumnDto[];
  onEditColumn?: (column: SchemaColumnDto) => void;
}

export function ColumnList({ columns, onEditColumn }: ColumnListProps) {
  const { t } = useTranslation();
  const [copiedCol, setCopiedCol] = useState<string | null>(null);

  if (columns.length === 0) {
    return <div className="p-4 text-sm text-[var(--app-text-muted)]">{t("schema.noColumns")}</div>;
  }

  const sorted = sortColumnsForDisplay(columns);

  const headerClass =
    "px-3 py-2 font-medium text-[12.5px] text-[var(--app-text-muted)] border-b border-[var(--app-border-subtle)]";

  const handleCopyName = async (name: string) => {
    await navigator.clipboard.writeText(name).catch(() => {});
    setCopiedCol(name);
    setTimeout(() => setCopiedCol(null), 1500);
  };

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
          {onEditColumn && <TableHead className={cn(headerClass, "w-10")} />}
        </TableRow>
      </TableHeader>
      <TableBody>
        {sorted.map((col) => (
          <TableRow
            key={col.name}
            className="group transition-colors hover:bg-[var(--app-hover)]"
          >
            <TableCell className="px-3 py-1.5 font-mono text-[13px]">
              <div className="flex items-center gap-1.5">
                <span className="select-text">{col.name}</span>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      className="h-5 w-5 shrink-0 p-0 opacity-0 transition-opacity group-hover:opacity-60 hover:!opacity-100 hover:bg-transparent"
                      onClick={() => handleCopyName(col.name)}
                    >
                      {copiedCol === col.name ? (
                        <Check className="h-3 w-3 text-emerald-500" />
                      ) : (
                        <Copy className="h-3 w-3" />
                      )}
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>Copy column name</TooltipContent>
                </Tooltip>
              </div>
            </TableCell>
            <TableCell className="px-3 py-1.5 font-mono text-[11px] text-[var(--app-text-muted)] select-text">
              {col.dataType}
            </TableCell>
            <TableCell className="px-3 py-1.5 text-[12px] text-[var(--app-text-muted)]">
              {col.nullable ? "YES" : "NO"}
            </TableCell>
            <TableCell className="px-3 py-1.5 font-mono text-[11px] text-[var(--app-text-muted)] select-text">
              {col.defaultValue ?? "\u2014"}
            </TableCell>
            <TableCell className="px-3 py-1.5 text-center">
              {col.isPrimaryKey && <Key className="inline h-3.5 w-3.5 text-primary" />}
            </TableCell>
            {onEditColumn && (
              <TableCell className="px-1 py-1.5 text-center">
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      className="h-6 w-6 p-0 opacity-0 transition-opacity group-hover:opacity-60 hover:!opacity-100"
                      onClick={() => onEditColumn(col)}
                    >
                      <Pencil className="h-3 w-3" />
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>Edit column</TooltipContent>
                </Tooltip>
              </TableCell>
            )}
          </TableRow>
        ))}
      </TableBody>
    </Table>
  );
}
