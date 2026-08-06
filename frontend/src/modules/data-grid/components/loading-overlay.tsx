import { useTranslation } from "@/commons/locales/useTranslation";

export function LoadingOverlay() {
  const { t } = useTranslation();
  return (
    <div
      className="absolute inset-0 z-10 flex items-center justify-center"
      style={{ backgroundColor: "rgba(0,0,0,0.08)" }}
    >
      <span className="text-sm text-muted-foreground">
        {t("common.states.loading")}
      </span>
    </div>
  );
}
