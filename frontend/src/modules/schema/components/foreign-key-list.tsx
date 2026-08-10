import { useTranslation } from "@/commons/locales/useTranslation";
import { useWorkspaceStore } from "@/commons/stores/workspace.store";
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
import { ArrowRight, ExternalLink } from "lucide-react";
import type { SchemaForeignKeyDto } from "../types/schema.types";
import { cn } from "@/lib/utils";

interface ForeignKeyListProps {
  foreignKeys: SchemaForeignKeyDto[];
  connectionId: string;
}

export function ForeignKeyList({ foreignKeys, connectionId }: ForeignKeyListProps) {
  const { t } = useTranslation();

  if (foreignKeys.length === 0) {
    return (
      <div className="p-4 text-sm text-[var(--app-text-muted)]">{t("schema.noForeignKeys")}</div>
    );
  }

  const openTargetTable = (schema: string, tableName: string) => {
    useWorkspaceStore.getState().openDbObject({
      id: `dbobj:${schema}.${tableName}:${connectionId}`,
      kind: "db-object",
      title: tableName,
      connectionId,
      resourceKey: `dbobj:${schema}.${tableName}:${connectionId}`,
      dirty: false,
      pinned: false,
      preview: false,
      order: Date.now(),
      data: {
        schema,
        objectName: tableName,
        objectType: "table",
        activeSection: "columns",
      },
    });
  };

  const headerClass =
    "px-3 py-2 font-medium text-[var(--app-text-muted)] border-b border-[var(--app-border-subtle)]";

  return (
    <Table className="w-full text-[13px]">
      <TableHeader>
        <TableRow>
          <TableHead className={cn(headerClass, "text-left")}>{t("schema.fkName")}</TableHead>
          <TableHead className={cn(headerClass, "text-left")}>{t("schema.fkFromColumn")}</TableHead>
          <TableHead className={cn(headerClass, "text-center")} />
          <TableHead className={cn(headerClass, "text-left")}>{t("schema.fkToTable")}</TableHead>
          <TableHead className={cn(headerClass, "text-left")}>{t("schema.fkToColumn")}</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {foreignKeys.map((fk) => (
          <TableRow key={fk.name} className="group transition-colors hover:bg-[var(--app-hover)]">
            <TableCell className="px-3 py-1.5 font-mono text-[13px] select-text">
              {fk.name}
            </TableCell>
            <TableCell className="px-3 py-1.5 font-mono text-[12px] text-[var(--app-text-muted)] select-text">
              {fk.fromColumn}
            </TableCell>
            <TableCell className="px-1 py-1.5 text-center">
              <ArrowRight className="inline h-3 w-3 text-[var(--app-text-muted)]" />
            </TableCell>
            <TableCell className="px-3 py-1.5">
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    type="button"
                    variant="ghost"
                    size="sm"
                    className="h-auto gap-1 p-0 font-mono text-[12px] text-primary hover:bg-transparent hover:underline"
                    onClick={() => openTargetTable(fk.toSchema, fk.toTable)}
                  >
                    {fk.toSchema !== fk.schema ? `${fk.toSchema}.` : ""}
                    {fk.toTable}
                    <ExternalLink className="h-3 w-3 opacity-50" />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>Open {fk.toSchema}.{fk.toTable}</TooltipContent>
              </Tooltip>
            </TableCell>
            <TableCell className="px-3 py-1.5 font-mono text-[12px] text-[var(--app-text-muted)] select-text">
              {fk.toColumn}
            </TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  );
}
