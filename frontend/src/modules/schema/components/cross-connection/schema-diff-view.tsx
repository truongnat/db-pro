import { useState } from "react";

import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
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
        <p className="text-[var(--text-secondary)]">{t("schema.crossConn.selectTwo")}</p>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center gap-3">
        <span className="text-sm text-[var(--text-secondary)]">{sourceLabel ?? sourceId}</span>
        <span className="text-[var(--text-secondary)]">→</span>
        <span className="text-sm text-[var(--text-secondary)]">{targetLabel ?? targetId}</span>
        <Button type="button" onClick={() => setEnabled(true)} disabled={isLoading}>
          {isLoading ? t("common.states.loading") : t("schema.crossConn.compare")}
        </Button>
      </div>

      {error && (
        <div className="rounded-sm bg-destructive px-3 py-2 text-sm text-white">
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
      <div className="rounded-sm border border-[var(--border-subtle)] p-4 text-center">
        <p className="text-success">{t("schema.crossConn.identical")}</p>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-4">
      {diff.tablesOnlyInSource.length > 0 && (
        <DiffSection title={t("schema.crossConn.onlyInSource")}>
          {diff.tablesOnlyInSource.map((t) => (
            <DiffItem key={t} label={t} color="text-destructive" />
          ))}
        </DiffSection>
      )}

      {diff.tablesOnlyInTarget.length > 0 && (
        <DiffSection title={t("schema.crossConn.onlyInTarget")}>
          {diff.tablesOnlyInTarget.map((t) => (
            <DiffItem key={t} label={t} color="text-success" />
          ))}
        </DiffSection>
      )}

      {diff.columnDiffs.length > 0 && (
        <DiffSection title={t("schema.crossConn.columnDifferences")}>
          {diff.columnDiffs.map((cd) => (
            <div key={`${cd.schema}.${cd.table}`} className="mb-2">
              <p className="text-sm font-medium text-foreground">
                {cd.schema}.{cd.table}
              </p>
              {cd.columnsOnlyInSource.map((c) => (
                <DiffItem key={c} label={`- ${c}`} color="text-destructive" />
              ))}
              {cd.columnsOnlyInTarget.map((c) => (
                <DiffItem key={c} label={`+ ${c}`} color="text-success" />
              ))}
              {cd.typeMismatches.map((m) => (
                <p key={m.column} className="pl-4 text-xs text-[var(--text-secondary)]">
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
            <DiffItem key={i} label={i} color="text-destructive" />
          ))}
        </DiffSection>
      )}

      {diff.indexesOnlyInTarget.length > 0 && (
        <DiffSection title={t("schema.crossConn.indexesOnlyTarget")}>
          {diff.indexesOnlyInTarget.map((i) => (
            <DiffItem key={i} label={i} color="text-success" />
          ))}
        </DiffSection>
      )}
    </div>
  );
}

function DiffSection({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="rounded-sm border border-[var(--border-subtle)] p-3">
      <h4 className="mb-2 text-sm font-medium text-foreground">{title}</h4>
      <div className="flex flex-col gap-1">{children}</div>
    </div>
  );
}

function DiffItem({ label, color }: { label: string; color: string }) {
  return <p className={cn("pl-2 text-xs font-mono", color)}>{label}</p>;
}
