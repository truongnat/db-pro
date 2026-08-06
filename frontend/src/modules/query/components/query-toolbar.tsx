import { useState } from "react";

import { useTranslation } from "@/commons/locales/useTranslation";

import { SQL_TEMPLATES } from "./sql-templates-data";

interface QueryToolbarProps {
  onExecute: () => void;
  onCancel: () => void;
  onExplain: () => void;
  onClear: () => void;
  onExport: () => void;
  onFormat: () => void;
  onInsertTemplate: (sql: string) => void;
  isExecuting: boolean;
  isExplaining: boolean;
  hasConnection: boolean;
  hasSql: boolean;
}

export function QueryToolbar({
  onExecute,
  onCancel,
  onExplain,
  onClear,
  onExport,
  onFormat,
  onInsertTemplate,
  isExecuting,
  isExplaining,
  hasConnection,
  hasSql,
}: QueryToolbarProps) {
  const { t } = useTranslation();
  const [templatesOpen, setTemplatesOpen] = useState(false);

  const primaryStyle: React.CSSProperties = {
    backgroundColor: "var(--color-primary,#3b82f6)",
    color: "white",
  };

  const secondaryStyle: React.CSSProperties = {
    borderColor: "var(--color-border)",
    color: "var(--color-text)",
  };

  return (
    <div
      className="flex items-center gap-2 border-b px-3 py-2"
      style={{
        borderColor: "var(--color-border)",
        backgroundColor: "var(--color-surface)",
      }}
    >
      <button
        className="rounded-[var(--radius-sm)] px-4 py-1.5 text-sm font-medium transition-colors disabled:opacity-50"
        style={primaryStyle}
        onClick={onExecute}
        disabled={!hasConnection || !hasSql || isExecuting}
        title="Ctrl+Enter"
      >
        {isExecuting ? t("common.states.loading") : t("query.execute")}
      </button>

      {isExecuting && (
        <button
          className="rounded-[var(--radius-sm)] px-3 py-1.5 text-sm font-medium transition-colors"
          style={{ backgroundColor: "var(--color-error,#ef4444)", color: "white" }}
          onClick={onCancel}
        >
          {t("common.actions.cancel")}
        </button>
      )}

      <button
        className="rounded-[var(--radius-sm)] border px-3 py-1.5 text-sm transition-colors disabled:opacity-50"
        style={secondaryStyle}
        onClick={onExplain}
        disabled={!hasConnection || !hasSql || isExplaining}
      >
        {isExplaining ? t("common.states.loading") : t("query.explain")}
      </button>

      <button
        className="rounded-[var(--radius-sm)] border px-3 py-1.5 text-sm transition-colors hover:bg-[var(--color-bg)] disabled:opacity-50"
        style={secondaryStyle}
        onClick={onExport}
        disabled={!hasConnection || !hasSql}
      >
        {t("export.title")}
      </button>

      <button
        className="rounded-[var(--radius-sm)] border px-3 py-1.5 text-sm transition-colors hover:bg-[var(--color-bg)] disabled:opacity-50"
        style={secondaryStyle}
        onClick={onFormat}
        disabled={!hasSql}
      >
        {t("query.format")}
      </button>

      <div style={{ position: "relative" }}>
        <button
          className="rounded-[var(--radius-sm)] border px-3 py-1.5 text-sm transition-colors hover:bg-[var(--color-bg)]"
          style={secondaryStyle}
          onClick={() => setTemplatesOpen((v) => !v)}
        >
          {t("query.templates")}
        </button>
        {templatesOpen && (
          <div
            className="absolute top-full left-0 z-50 mt-1 max-h-64 w-56 overflow-auto rounded-[var(--radius-sm)] border py-1 shadow-lg"
            style={{
              borderColor: "var(--color-border)",
              backgroundColor: "var(--color-surface)",
            }}
          >
            {SQL_TEMPLATES.map((tpl) => (
              <button
                key={tpl.label}
                className="block w-full px-3 py-1.5 text-left text-sm transition-colors hover:bg-[var(--color-bg)]"
                onClick={() => {
                  onInsertTemplate(tpl.sql);
                  setTemplatesOpen(false);
                }}
              >
                {tpl.label}
              </button>
            ))}
          </div>
        )}
      </div>

      <div className="flex-1" />

      <button
        className="rounded-[var(--radius-sm)] border px-3 py-1.5 text-sm transition-colors hover:bg-[var(--color-bg)]"
        style={secondaryStyle}
        onClick={onClear}
      >
        {t("query.clear")}
      </button>
    </div>
  );
}
