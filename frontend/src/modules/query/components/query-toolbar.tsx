import { useState } from "react";

import { Button } from "@/components/ui/button";
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

  return (
    <div className="flex items-center gap-2 border-b border-border bg-card px-3 py-2">
      <Button
        type="button"
        className="rounded-sm px-4 py-1.5 text-sm font-medium"
        onClick={onExecute}
        disabled={!hasConnection || !hasSql || isExecuting}
        title="Ctrl+Enter"
      >
        {isExecuting ? t("common.states.loading") : t("query.execute")}
      </Button>

      {isExecuting && (
        <Button
          type="button"
          className="rounded-sm bg-destructive px-3 py-1.5 text-sm font-medium text-white hover:bg-destructive/90"
          onClick={onCancel}
        >
          {t("common.actions.cancel")}
        </Button>
      )}

      <Button
        type="button"
        variant="outline"
        className="rounded-sm border px-3 py-1.5 text-sm"
        onClick={onExplain}
        disabled={!hasConnection || !hasSql || isExplaining}
      >
        {isExplaining ? t("common.states.loading") : t("query.explain")}
      </Button>

      <Button
        type="button"
        variant="outline"
        className="rounded-sm border px-3 py-1.5 text-sm"
        onClick={onExport}
        disabled={!hasConnection || !hasSql}
      >
        {t("export.title")}
      </Button>

      <Button
        type="button"
        variant="outline"
        className="rounded-sm border px-3 py-1.5 text-sm"
        onClick={onFormat}
        disabled={!hasSql}
      >
        {t("query.format")}
      </Button>

      <div style={{ position: "relative" }}>
        <Button
          type="button"
          variant="outline"
          className="rounded-sm border px-3 py-1.5 text-sm"
          onClick={() => setTemplatesOpen((v) => !v)}
        >
          {t("query.templates")}
        </Button>
        {templatesOpen && (
          <div className="absolute top-full left-0 z-50 mt-1 max-h-64 w-56 overflow-auto rounded-sm border border-border bg-card py-1 shadow-lg">
            {SQL_TEMPLATES.map((tpl) => (
              <Button
                key={tpl.label}
                type="button"
                variant="ghost"
                className="w-full justify-start rounded-none border-0 px-3 py-1.5 text-left text-sm"
                onClick={() => {
                  onInsertTemplate(tpl.sql);
                  setTemplatesOpen(false);
                }}
              >
                {tpl.label}
              </Button>
            ))}
          </div>
        )}
      </div>

      <div className="flex-1" />

      <Button
        type="button"
        variant="outline"
        className="rounded-sm border px-3 py-1.5 text-sm"
        onClick={onClear}
      >
        {t("query.clear")}
      </Button>
    </div>
  );
}
