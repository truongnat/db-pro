import { useTranslation } from "@/commons/locales/useTranslation";

export function UsersView() {
  const { t } = useTranslation();

  return (
    <div className="flex min-h-0 flex-col gap-2 px-2">
      <span className="text-[10px] font-semibold uppercase tracking-widest text-[var(--app-text-dim)]">
        {t("userManagement.title")}
      </span>
      <p className="py-1 text-xs text-[var(--app-text-dim)]">
        {t("userManagement.sidebarHint")}
      </p>
    </div>
  );
}
