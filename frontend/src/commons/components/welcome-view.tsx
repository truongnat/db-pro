import { useTranslation } from "@/commons/locales/useTranslation";
import { useConnectionStore } from "@/commons/stores/connection.store";

export function WelcomeView() {
  const { t } = useTranslation();
  const activeConnectionId = useConnectionStore((s) => s.activeConnectionId);

  return (
    <div className="flex flex-1 flex-col items-center justify-center gap-4 text-sm text-muted-foreground">
      {!activeConnectionId ? (
        <p>{t("workspace.connectToStart")}</p>
      ) : (
        <div className="flex flex-col items-center gap-2">
          <p>{t("workspace.noTabsOpen")}</p>
          <p className="text-xs">
            {t("workspace.hintSidebar")}
          </p>
        </div>
      )}
    </div>
  );
}
