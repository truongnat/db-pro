import { useTranslation } from "@/commons/locales/useTranslation";

import { useListTablespaces } from "../../queries/schema.queries";

interface TablespaceListProps {
  connectionId: string | null;
}

export function TablespaceList({ connectionId }: TablespaceListProps) {
  const { t } = useTranslation();
  const { data: tablespaces, isLoading, error } = useListTablespaces(connectionId, !!connectionId);

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

  if (!tablespaces?.length) {
    return (
      <div className="flex items-center justify-center py-12">
        <p style={{ color: "var(--color-text-secondary)" }}>{t("schema.crossConn.noTablespaces")}</p>
      </div>
    );
  }

  return (
    <div className="overflow-hidden rounded-[var(--radius-md)] border" style={{ borderColor: "var(--color-border)" }}>
      <table className="w-full text-sm">
        <thead>
          <tr style={{ backgroundColor: "var(--color-surface)" }}>
            <th className="px-4 py-3 text-left font-medium" style={{ color: "var(--color-text-secondary)" }}>
              {t("common.labels.name")}
            </th>
            <th className="px-4 py-3 text-left font-medium" style={{ color: "var(--color-text-secondary)" }}>
              {t("schema.crossConn.owner")}
            </th>
            <th className="px-4 py-3 text-left font-medium" style={{ color: "var(--color-text-secondary)" }}>
              {t("schema.crossConn.location")}
            </th>
          </tr>
        </thead>
        <tbody>
          {tablespaces.map((ts) => (
            <tr key={ts.name} className="border-t" style={{ borderColor: "var(--color-border)" }}>
              <td className="px-4 py-3 font-medium" style={{ color: "var(--color-text)" }}>{ts.name}</td>
              <td className="px-4 py-3" style={{ color: "var(--color-text-secondary)" }}>{ts.owner}</td>
              <td className="px-4 py-3 font-mono text-xs" style={{ color: "var(--color-text-secondary)" }}>{ts.location || "(default)"}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
