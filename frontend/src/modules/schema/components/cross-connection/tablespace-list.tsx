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

  if (!tablespaces?.length) {
    return (
      <div className="flex items-center justify-center py-12">
        <p className="text-muted-foreground">{t("schema.crossConn.noTablespaces")}</p>
      </div>
    );
  }

  return (
    <div className="overflow-hidden rounded-md border border-border">
      <table className="w-full text-sm">
        <thead>
          <tr className="bg-card">
            <th className="px-4 py-3 text-left font-medium text-muted-foreground">
              {t("common.labels.name")}
            </th>
            <th className="px-4 py-3 text-left font-medium text-muted-foreground">
              {t("schema.crossConn.owner")}
            </th>
            <th className="px-4 py-3 text-left font-medium text-muted-foreground">
              {t("schema.crossConn.location")}
            </th>
          </tr>
        </thead>
        <tbody>
          {tablespaces.map((ts) => (
            <tr key={ts.name} className="border-t border-border">
              <td className="px-4 py-3 font-medium text-foreground">{ts.name}</td>
              <td className="px-4 py-3 text-muted-foreground">{ts.owner}</td>
              <td className="px-4 py-3 font-mono text-xs text-muted-foreground">{ts.location || "(default)"}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
