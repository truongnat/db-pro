import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { useTranslation } from "@/commons/locales/useTranslation";

import { useListPartitions } from "../../queries/schema.queries";

interface PartitionManagerProps {
  connectionId: string | null;
}

export function PartitionManager({ connectionId }: PartitionManagerProps) {
  const { t } = useTranslation();
  const { data: partitions, isLoading, error } = useListPartitions(connectionId, !!connectionId);

  if (!connectionId) {
    return (
      <div className="flex items-center justify-center py-12">
        <p className="text-[var(--app-text-muted)]">
          {t("schema.connectFirst")}
        </p>
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <p className="text-[var(--app-text-muted)]">{t("common.states.loading")}</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="rounded-sm bg-destructive px-3 py-2 text-sm text-white">
        {(error as Error).message}
      </div>
    );
  }

  if (!partitions?.length) {
    return (
      <div className="flex items-center justify-center py-12">
        <p className="text-[var(--app-text-muted)]">{t("schema.crossConn.noPartitions")}</p>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-4">
      {partitions.map((p) => (
        <div key={`${p.schema}.${p.table}`} className="rounded-sm border border-[var(--app-border)]">
          <div className="flex items-center gap-2 border-b border-[var(--app-border-subtle)] bg-background px-4 py-3">
            <span className="font-medium text-foreground">
              {p.schema}.{p.table}
            </span>
            <span className="rounded bg-background px-1.5 py-0.5 text-xs text-[var(--app-text-muted)]">
              {p.partitionStrategy}
            </span>
          </div>
          {p.partitions.length > 0 && (
            <Table className="w-full text-sm">
              <TableHeader>
                <TableRow className="bg-background">
                  <TableHead className="px-4 py-2 text-left font-medium text-[var(--app-text-muted)]">
                    {t("schema.crossConn.partitionName")}
                  </TableHead>
                  <TableHead className="px-4 py-2 text-left font-medium text-[var(--app-text-muted)]">
                    {t("schema.crossConn.boundExpr")}
                  </TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {p.partitions.map((child) => (
                  <TableRow key={child.name} className="border-t border-[var(--app-border-subtle)]">
                    <TableCell className="px-4 py-2 text-foreground">{child.name}</TableCell>
                    <TableCell className="px-4 py-2 font-mono text-xs text-[var(--app-text-muted)]">{child.boundExpr}</TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </div>
      ))}
    </div>
  );
}
