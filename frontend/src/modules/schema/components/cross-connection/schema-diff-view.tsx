import { useState } from "react";

import { useTranslation } from "@/commons/locales/useTranslation";

import { useDiffSchemas } from "../../queries/schema.queries";
import type { SchemaDiff } from "../../types/schema.types";

interface SchemaDiffViewProps {
  sourceId: string | null;
  targetId: string | null;
  sourceLabel?: string;
  targetLabel?: string;
}

export function SchemaDiffView({
  sourceId,
  targetId,
  sourceLabel,
  targetLabel,
}: SchemaDiffViewProps) {
  const { t } = useTranslation();
  const [enabled, setEnabled] = useState(false);
  const { data: diff, isLoading, error } = useDiffSchemas(sourceId, targetId, enabled);

  if (!sourceId || !targetId) {
    return (
      <div className="flex items-center justify-center py-12">
        <p style={{ color: "var(--color-text-secondary)" }}>
          {t("schema.crossConn.selectTwo")}
        </p>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center gap-3">
        <span className="text-sm" style={{ color: "var(--color-text-secondary)" }}>
          {sourceLabel ?? sourceId}
        </span>
        <span style={{ color: "var(--color-text-secondary)" }}>→</span>
        <span className="text-sm" style={{ color: "var(--color-text-secondary)" }}>
          {targetLabel ?? targetId}
        </span>
        <button
          className="rounded-[var(--radius-sm)] px-3 py-1.5 text-sm text-white"
          style={{ backgroundColor: "var(--color-primary,#3b82f6)" }}
          onClick={() => setEnabled(true)}
          disabled={isLoading}
        >
          {isLoading ? t("common.states.loading") : t("schema.crossConn.compare")}
        </button>
      </div>

      {error && (
        <div className="rounded-[var(--radius-sm)] px-3 py-2 text-sm" style={{ backgroundColor: "var(--color-error,#ef4444)", color: "white" }}>
          {(error as Error).message}
        </div>
      )}

      {diff && <DiffResults diff={diff} />}
    </div>
  );
}

function DiffResults({ diff }: { diff: SchemaDiff }) {
  const { t } = useTranslation();
  const hasDiffs =
    diff.tablesOnlyInSource.length > 0 ||
    diff.tablesOnlyInTarget.length > 0 ||
    diff.columnDiffs.length > 0 ||
    diff.indexesOnlyInSource.length > 0 ||
    diff.indexesOnlyInTarget.length > 0;

  if (!hasDiffs) {
    return (
      <div className="rounded-[var(--radius-sm)] border p-4 text-center" style={{ borderColor: "var(--color-border)" }}>
        <p style={{ color: "var(--color-success,#22c55e)" }}>{t("schema.crossConn.identical")}</p>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-4">
      {diff.tablesOnlyInSource.length > 0 && (
        <DiffSection title={t("schema.crossConn.onlyInSource")}>
          {diff.tablesOnlyInSource.map((t) => (
            <DiffItem key={t} label={t} color="var(--color-error,#ef4444)" />
          ))}
        </DiffSection>
      )}

      {diff.tablesOnlyInTarget.length > 0 && (
        <DiffSection title={t("schema.crossConn.onlyInTarget")}>
          {diff.tablesOnlyInTarget.map((t) => (
            <DiffItem key={t} label={t} color="var(--color-success,#22c55e)" />
          ))}
        </DiffSection>
      )}

      {diff.columnDiffs.length > 0 && (
        <DiffSection title={t("schema.crossConn.columnDifferences")}>
          {diff.columnDiffs.map((cd) => (
            <div key={`${cd.schema}.${cd.table}`} className="mb-2">
              <p className="text-sm font-medium" style={{ color: "var(--color-text)" }}>
                {cd.schema}.{cd.table}
              </p>
              {cd.columnsOnlyInSource.map((c) => (
                <DiffItem key={c} label={`- ${c}`} color="var(--color-error,#ef4444)" />
              ))}
              {cd.columnsOnlyInTarget.map((c) => (
                <DiffItem key={c} label={`+ ${c}`} color="var(--color-success,#22c55e)" />
              ))}
              {cd.typeMismatches.map((m) => (
                <p key={m.column} className="pl-4 text-xs" style={{ color: "var(--color-text-secondary)" }}>
                  {m.column}: {m.sourceType} → {m.targetType}
                </p>
              ))}
            </div>
          ))}
        </DiffSection>
      )}

      {diff.indexesOnlyInSource.length > 0 && (
        <DiffSection title={t("schema.crossConn.indexesOnlySource")}>
          {diff.indexesOnlyInSource.map((i) => (
            <DiffItem key={i} label={i} color="var(--color-error,#ef4444)" />
          ))}
        </DiffSection>
      )}

      {diff.indexesOnlyInTarget.length > 0 && (
        <DiffSection title={t("schema.crossConn.indexesOnlyTarget")}>
          {diff.indexesOnlyInTarget.map((i) => (
            <DiffItem key={i} label={i} color="var(--color-success,#22c55e)" />
          ))}
        </DiffSection>
      )}
    </div>
  );
}

function DiffSection({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="rounded-[var(--radius-sm)] border p-3" style={{ borderColor: "var(--color-border)" }}>
      <h4 className="mb-2 text-sm font-medium" style={{ color: "var(--color-text)" }}>{title}</h4>
      <div className="flex flex-col gap-1">{children}</div>
    </div>
  );
}

function DiffItem({ label, color }: { label: string; color: string }) {
  return (
    <p className="pl-2 text-xs font-mono" style={{ color }}>{label}</p>
  );
}
