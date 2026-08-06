import { useState } from "react";

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
        <p style={{ color: "var(--color-text-secondary)" }}>
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
          className="rounded-[var(--radius-sm)] border px-3 py-2 text-sm"
          style={{ borderColor: "var(--color-border)", color: "var(--color-text)" }}
        />
        <input
          type="text"
          value={objectName}
          onChange={(e) => setObjectName(e.target.value)}
          placeholder={t("schema.crossConn.objectName")}
          className="rounded-[var(--radius-sm)] border px-3 py-2 text-sm"
          style={{ borderColor: "var(--color-border)", color: "var(--color-text)" }}
        />
        <button
          className="rounded-[var(--radius-sm)] px-3 py-2 text-sm text-white"
          style={{ backgroundColor: "var(--color-primary,#3b82f6)" }}
          onClick={() => setEnabled(true)}
          disabled={isLoading || !schema || !objectName}
        >
          {isLoading ? t("common.states.loading") : t("schema.crossConn.viewDeps")}
        </button>
      </div>

      {error && (
        <div className="rounded-[var(--radius-sm)] px-3 py-2 text-sm" style={{ backgroundColor: "var(--color-error,#ef4444)", color: "white" }}>
          {(error as Error).message}
        </div>
      )}

      {deps && deps.length === 0 && (
        <p className="text-sm" style={{ color: "var(--color-text-secondary)" }}>
          {t("schema.crossConn.noDeps")}
        </p>
      )}

      {deps && deps.length > 0 && (
        <div className="rounded-[var(--radius-sm)] border" style={{ borderColor: "var(--color-border)" }}>
          <table className="w-full text-sm">
            <thead>
              <tr style={{ backgroundColor: "var(--color-surface)" }}>
                <th className="px-3 py-2 text-left font-medium" style={{ color: "var(--color-text-secondary)" }}>
                  {t("schema.crossConn.object")}
                </th>
                <th className="px-3 py-2 text-left font-medium" style={{ color: "var(--color-text-secondary)" }}>
                  {t("schema.crossConn.dependsOn")}
                </th>
              </tr>
            </thead>
            <tbody>
              {deps.map((dep, i) => (
                <tr key={i} className="border-t" style={{ borderColor: "var(--color-border)" }}>
                  <td className="px-3 py-2">
                    <span className="rounded px-1.5 py-0.5 text-xs" style={{ backgroundColor: "var(--color-bg)", color: "var(--color-text-secondary)" }}>
                      {dep.objectType}
                    </span>
                    <span className="ml-2" style={{ color: "var(--color-text)" }}>{dep.objectName}</span>
                  </td>
                  <td className="px-3 py-2">
                    <span className="rounded px-1.5 py-0.5 text-xs" style={{ backgroundColor: "var(--color-bg)", color: "var(--color-text-secondary)" }}>
                      {dep.dependsOnType}
                    </span>
                    <span className="ml-2" style={{ color: "var(--color-text)" }}>{dep.dependsOnName}</span>
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
