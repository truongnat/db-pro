import { useState } from "react";

import { Button } from "@/components/ui/button";
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
        <p className="text-muted-foreground">
          {t("schema.connectFirst")}
        </p>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-4">
      <div className="grid grid-cols-3 gap-3">
        <input
          type="text"
          value={schema}
          onChange={(e) => setSchema(e.target.value)}
          placeholder={t("schema.crossConn.schemaName")}
          className="rounded-sm border border-border px-3 py-2 text-sm text-foreground"
        />
        <input
          type="text"
          value={objectName}
          onChange={(e) => setObjectName(e.target.value)}
          placeholder={t("schema.crossConn.objectName")}
          className="rounded-sm border border-border px-3 py-2 text-sm text-foreground"
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
        <p className="text-sm text-muted-foreground">
          {t("schema.crossConn.noDeps")}
        </p>
      )}

      {deps && deps.length > 0 && (
        <div className="rounded-sm border border-border">
          <table className="w-full text-sm">
            <thead>
              <tr className="bg-card">
                <th className="px-3 py-2 text-left font-medium text-muted-foreground">
                  {t("schema.crossConn.object")}
                </th>
                <th className="px-3 py-2 text-left font-medium text-muted-foreground">
                  {t("schema.crossConn.dependsOn")}
                </th>
              </tr>
            </thead>
            <tbody>
              {deps.map((dep, i) => (
                <tr key={i} className="border-t border-border">
                  <td className="px-3 py-2">
                    <span className="rounded px-1.5 py-0.5 text-xs bg-background text-muted-foreground">
                      {dep.objectType}
                    </span>
                    <span className="ml-2 text-foreground">{dep.objectName}</span>
                  </td>
                  <td className="px-3 py-2">
                    <span className="rounded px-1.5 py-0.5 text-xs bg-background text-muted-foreground">
                      {dep.dependsOnType}
                    </span>
                    <span className="ml-2 text-foreground">{dep.dependsOnName}</span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
