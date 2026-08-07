import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
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
    <div className="overflow-hidden rounded-md border border-[var(--app-border)]">
      <Table className="w-full text-sm">
        <TableHeader>
          <TableRow className="bg-background">
            <TableHead className="px-4 py-3 text-left font-medium text-[var(--app-text-muted)]">
              {t("common.labels.name")}
            </TableHead>
            <TableHead className="px-4 py-3 text-left font-medium text-[var(--app-text-muted)]">
              {t("schema.crossConn.owner")}
            </TableHead>
            <TableHead className="px-4 py-3 text-left font-medium text-[var(--app-text-muted)]">
              {t("schema.crossConn.location")}
            </TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {tablespaces.map((ts) => (
            <TableRow key={ts.name} className="border-t border-[var(--app-border-subtle)]">
              <TableCell className="px-4 py-3 font-medium text-foreground">{ts.name}</TableCell>
              <TableCell className="px-4 py-3 text-[var(--app-text-muted)]">{ts.owner}</TableCell>
              <TableCell className="px-4 py-3 font-mono text-xs text-[var(--app-text-muted)]">{ts.location || "(default)"}</TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  );
}
