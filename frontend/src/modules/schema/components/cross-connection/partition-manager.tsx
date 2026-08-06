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
        <p className="text-muted-foreground">
          {t("schema.connectFirst")}
        </p>
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <p className="text-muted-foreground">{t("common.states.loading")}</p>
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
        <p className="text-muted-foreground">{t("schema.crossConn.noPartitions")}</p>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-4">
      {partitions.map((p) => (
        <div key={`${p.schema}.${p.table}`} className="rounded-sm border border-border">
          <div className="flex items-center gap-2 border-b border-border bg-card px-4 py-3">
            <span className="font-medium text-foreground">
              {p.schema}.{p.table}
            </span>
            <span className="rounded bg-background px-1.5 py-0.5 text-xs text-muted-foreground">
              {p.partitionStrategy}
            </span>
          </div>
          {p.partitions.length > 0 && (
            <table className="w-full text-sm">
              <thead>
                <tr className="bg-background">
                  <th className="px-4 py-2 text-left font-medium text-muted-foreground">
                    {t("schema.crossConn.partitionName")}
                  </th>
                  <th className="px-4 py-2 text-left font-medium text-muted-foreground">
                    {t("schema.crossConn.boundExpr")}
                  </th>
                </tr>
              </thead>
              <tbody>
                {p.partitions.map((child) => (
                  <tr key={child.name} className="border-t border-border">
                    <td className="px-4 py-2 text-foreground">{child.name}</td>
                    <td className="px-4 py-2 font-mono text-xs text-muted-foreground">{child.boundExpr}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      ))}
    </div>
  );
}
