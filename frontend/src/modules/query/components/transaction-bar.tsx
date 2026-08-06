import { useTranslation } from "@/commons/locales/useTranslation";

export function TransactionBar() {
  const { t } = useTranslation();

  const buttonClass =
    "rounded-[var(--radius-sm)] border px-3 py-1 text-xs transition-colors disabled:opacity-50";
  const buttonStyle: React.CSSProperties = {
    borderColor: "var(--color-border)",
    color: "var(--color-text)",
  };

  return (
    <div
      className="flex items-center gap-2 border-b px-3 py-1.5"
      style={{
        borderColor: "var(--color-border)",
        backgroundColor: "var(--color-surface)",
      }}
    >
      <button
        className={buttonClass}
        style={buttonStyle}
        disabled
        title={t("query.transactionComingSoon")}
      >
        {t("query.begin")}
      </button>
      <button
        className={buttonClass}
        style={buttonStyle}
        disabled
        title={t("query.transactionComingSoon")}
      >
        {t("query.commit")}
      </button>
      <button
        className={buttonClass}
        style={buttonStyle}
        disabled
        title={t("query.transactionComingSoon")}
      >
        {t("query.rollback")}
      </button>
    </div>
  );
}
