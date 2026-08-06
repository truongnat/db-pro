import { Button } from "@/components/ui/button";
import { useTranslation } from "@/commons/locales/useTranslation";

export function TransactionBar() {
  const { t } = useTranslation();

  return (
    <div className="flex items-center gap-2 border-b border-border bg-card px-3 py-1.5">
      <Button
        type="button"
        variant="outline"
        className="rounded-sm border px-3 py-1 text-xs"
        disabled
        title={t("query.transactionComingSoon")}
      >
        {t("query.begin")}
      </Button>
      <Button
        type="button"
        variant="outline"
        className="rounded-sm border px-3 py-1 text-xs"
        disabled
        title={t("query.transactionComingSoon")}
      >
        {t("query.commit")}
      </Button>
      <Button
        type="button"
        variant="outline"
        className="rounded-sm border px-3 py-1 text-xs"
        disabled
        title={t("query.transactionComingSoon")}
      >
        {t("query.rollback")}
      </Button>
    </div>
  );
}
