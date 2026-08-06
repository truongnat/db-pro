import { useTranslation } from "@/commons/locales/useTranslation";

interface SchemaToolbarProps {
  searchQuery: string;
  onSearchChange: (query: string) => void;
  onRefresh: () => void;
  isRefreshing: boolean;
  tableCount: number;
}

export function SchemaToolbar({
  searchQuery,
  onSearchChange,
  onRefresh,
  isRefreshing,
  tableCount,
}: SchemaToolbarProps) {
  const { t } = useTranslation();

  const inputStyle: React.CSSProperties = {
    borderColor: "var(--color-border)",
    backgroundColor: "var(--color-bg)",
    color: "var(--color-text)",
  };

  const buttonStyle: React.CSSProperties = {
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
      <input
        type="text"
        className="flex-1 rounded-[var(--radius-sm)] border px-2 py-1 text-sm outline-none"
        style={inputStyle}
        placeholder={t("schema.searchPlaceholder")}
        value={searchQuery}
        onChange={(e) => onSearchChange(e.target.value)}
      />

      <span
        className="text-xs"
        style={{ color: "var(--color-text-secondary)" }}
      >
        {tableCount}
      </span>

      <button
        className="rounded-[var(--radius-sm)] border px-3 py-1 text-sm transition-colors hover:bg-[var(--color-bg)] disabled:opacity-50"
        style={buttonStyle}
        onClick={onRefresh}
        disabled={isRefreshing}
      >
        {isRefreshing ? t("common.states.loading") : t("common.actions.refresh")}
      </button>
    </div>
  );
}
