import { useTranslation } from "@/commons/locales/useTranslation";

interface QueryToolbarProps {
  onExecute: () => void;
  onExplain: () => void;
  onClear: () => void;
  isExecuting: boolean;
  isExplaining: boolean;
  hasConnection: boolean;
  hasSql: boolean;
}

export function QueryToolbar({
  onExecute,
  onExplain,
  onClear,
  isExecuting,
  isExplaining,
  hasConnection,
  hasSql,
}: QueryToolbarProps) {
  const { t } = useTranslation();

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

      <button
        className="rounded-[var(--radius-sm)] border px-3 py-1.5 text-sm transition-colors disabled:opacity-50"
        style={secondaryStyle}
        onClick={onExplain}
        disabled={!hasConnection || !hasSql || isExplaining}
      >
        {isExplaining ? t("common.states.loading") : t("query.explain")}
      </button>

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
