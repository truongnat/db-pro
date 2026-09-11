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
import { groupForeignKeys } from "../utils/foreign-key-groups";
import { cn } from "@/lib/utils";

interface ForeignKeyListProps {
  foreignKeys: SchemaForeignKeyDto[];
  connectionId: string;
}

export function ForeignKeyList({ foreignKeys, connectionId }: ForeignKeyListProps) {
  const { t } = useTranslation();
  const relations = groupForeignKeys(foreignKeys);

  if (relations.length === 0) {
    return (
      <div className="p-4 text-sm text-[var(--text-secondary)]">{t("schema.noForeignKeys")}</div>
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
    "px-3 py-2 font-medium text-[var(--text-secondary)] border-b border-[var(--border-subtle)]";

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
        {relations.map((relation) => (
          <TableRow
            key={relation.key}
            className="group transition-colors hover:bg-[var(--surface-hover)]"
          >
            <TableCell className="px-3 py-1.5 font-mono text-[13px] select-text">
              {relation.name}
            </TableCell>
            <TableCell className="px-3 py-1.5 font-mono text-[12px] text-[var(--text-secondary)] select-text">
              {relation.fromColumns.join(", ")}
            </TableCell>
            <TableCell className="px-1 py-1.5 text-center">
              <ArrowRight className="inline h-3 w-3 text-[var(--text-secondary)]" />
            </TableCell>
            <TableCell className="px-3 py-1.5">
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    type="button"
                    variant="ghost"
                    size="sm"
                    className="h-auto gap-1 p-0 font-mono text-[12px] text-primary hover:bg-transparent hover:underline"
                    onClick={() => openTargetTable(relation.toSchema, relation.toTable)}
                  >
                    {relation.toSchema !== relation.schema ? `${relation.toSchema}.` : ""}
                    {relation.toTable}
                    <ExternalLink className="h-3 w-3 opacity-50" />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>
                  Open {relation.toSchema}.{relation.toTable}
                </TooltipContent>
              </Tooltip>
            </TableCell>
            <TableCell className="px-3 py-1.5 font-mono text-[12px] text-[var(--text-secondary)] select-text">
              {relation.toColumns.join(", ")}
            </TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  );
}
