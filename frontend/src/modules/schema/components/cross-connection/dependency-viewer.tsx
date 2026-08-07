import { useState } from "react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { useTranslation } from "@/commons/locales/useTranslation";

import { useObjectDependencies } from "../../queries/schema.queries";

interface DependencyViewerProps {
  connectionId: string | null;
}

export function DependencyViewer({ connectionId }: DependencyViewerProps) {
  const { t } = useTranslation();
  const [schema, setSchema] = useState("");
  const [objectName, setObjectName] = useState("");
  const [enabled, setEnabled] = useState(false);
  const { data: deps, isLoading, error } = useObjectDependencies(
    connectionId,
    schema || null,
    objectName || null,
    enabled,
  );

  if (!connectionId) {
    return (
      <div className="flex items-center justify-center py-12">
        <p className="text-[var(--app-text-muted)]">
          {t("schema.connectFirst")}
        </p>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-4">
      <div className="grid grid-cols-3 gap-3">
        <Input
          value={schema}
          onChange={(e) => setSchema(e.target.value)}
          placeholder={t("schema.crossConn.schemaName")}
          className="rounded-sm border border-[var(--app-border)] px-3 py-2 text-sm text-foreground"
        />
        <Input
          value={objectName}
          onChange={(e) => setObjectName(e.target.value)}
          placeholder={t("schema.crossConn.objectName")}
          className="rounded-sm border border-[var(--app-border)] px-3 py-2 text-sm text-foreground"
        />
        <Button
          type="button"
          onClick={() => setEnabled(true)}
          disabled={isLoading || !schema || !objectName}
        >
          {isLoading ? t("common.states.loading") : t("schema.crossConn.viewDeps")}
        </Button>
      </div>

      {error && (
        <div className="rounded-sm bg-destructive px-3 py-2 text-sm text-white">
          {(error as Error).message}
        </div>
      )}

      {deps && deps.length === 0 && (
        <p className="text-sm text-[var(--app-text-muted)]">
          {t("schema.crossConn.noDeps")}
        </p>
      )}

      {deps && deps.length > 0 && (
        <div className="rounded-sm border border-[var(--app-border)]">
          <Table className="w-full text-sm">
            <TableHeader>
              <TableRow className="bg-background">
                <TableHead className="px-3 py-2 text-left font-medium text-[var(--app-text-muted)]">
                  {t("schema.crossConn.object")}
                </TableHead>
                <TableHead className="px-3 py-2 text-left font-medium text-[var(--app-text-muted)]">
                  {t("schema.crossConn.dependsOn")}
                </TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {deps.map((dep, i) => (
                <TableRow key={i} className="border-t border-[var(--app-border-subtle)]">
                  <TableCell className="px-3 py-2">
                    <span className="rounded px-1.5 py-0.5 text-xs bg-background text-[var(--app-text-muted)]">
                      {dep.objectType}
                    </span>
                    <span className="ml-2 text-foreground">{dep.objectName}</span>
                  </TableCell>
                  <TableCell className="px-3 py-2">
                    <span className="rounded px-1.5 py-0.5 text-xs bg-background text-[var(--app-text-muted)]">
                      {dep.dependsOnType}
                    </span>
                    <span className="ml-2 text-foreground">{dep.dependsOnName}</span>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      )}
    </div>
  );
}
