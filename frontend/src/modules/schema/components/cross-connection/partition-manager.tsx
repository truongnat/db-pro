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
        <p style={{ color: "var(--color-text-secondary)" }}>
          {t("schema.connectFirst")}
        </p>
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <p style={{ color: "var(--color-text-secondary)" }}>{t("common.states.loading")}</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="rounded-[var(--radius-sm)] px-3 py-2 text-sm" style={{ backgroundColor: "var(--color-error,#ef4444)", color: "white" }}>
        {(error as Error).message}
      </div>
    );
  }

  if (!partitions?.length) {
    return (
      <div className="flex items-center justify-center py-12">
        <p style={{ color: "var(--color-text-secondary)" }}>{t("schema.crossConn.noPartitions")}</p>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-4">
      {partitions.map((p) => (
        <div key={`${p.schema}.${p.table}`} className="rounded-[var(--radius-sm)] border" style={{ borderColor: "var(--color-border)" }}>
          <div className="flex items-center gap-2 border-b px-4 py-3" style={{ borderColor: "var(--color-border)", backgroundColor: "var(--color-surface)" }}>
            <span className="font-medium" style={{ color: "var(--color-text)" }}>
              {p.schema}.{p.table}
            </span>
            <span className="rounded px-1.5 py-0.5 text-xs" style={{ backgroundColor: "var(--color-bg)", color: "var(--color-text-secondary)" }}>
              {p.partitionStrategy}
            </span>
          </div>
          {p.partitions.length > 0 && (
            <table className="w-full text-sm">
              <thead>
                <tr style={{ backgroundColor: "var(--color-bg)" }}>
                  <th className="px-4 py-2 text-left font-medium" style={{ color: "var(--color-text-secondary)" }}>
                    {t("schema.crossConn.partitionName")}
                  </th>
                  <th className="px-4 py-2 text-left font-medium" style={{ color: "var(--color-text-secondary)" }}>
                    {t("schema.crossConn.boundExpr")}
                  </th>
                </tr>
              </thead>
              <tbody>
                {p.partitions.map((child) => (
                  <tr key={child.name} className="border-t" style={{ borderColor: "var(--color-border)" }}>
                    <td className="px-4 py-2" style={{ color: "var(--color-text)" }}>{child.name}</td>
                    <td className="px-4 py-2 font-mono text-xs" style={{ color: "var(--color-text-secondary)" }}>{child.boundExpr}</td>
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
