import { useTranslation } from "@/commons/locales/useTranslation";
import { useConnectionStore } from "@/commons/stores/connection.store";
import { useConnectionList } from "@/modules/connection/queries/connection.queries";

export function UsersView() {
  const { t } = useTranslation();
  const connections = useConnectionList();
  const explorerConnectionId = useConnectionStore((s) => s.explorerConnectionId);
  const selectedConnection = connections.data?.find((c) => c.id === explorerConnectionId);

  return (
    <div className="flex min-h-0 flex-col gap-2 px-2">
      <span className="text-[11px] font-semibold uppercase tracking-widest text-[var(--text-tertiary)]">
        {t("userManagement.title")}
      </span>
      {!selectedConnection ? (
        <p className="py-1 text-xs text-[var(--text-tertiary)]">
          {t("userManagement.connectFirst")}
        </p>
      ) : selectedConnection.driver === "sqlite" ? (
        <div className="rounded-md border border-[var(--border-subtle)] bg-[var(--surface-panel)] p-2.5">
          <p className="text-xs font-medium text-foreground">{t("userManagement.postgresOnly")}</p>
          <p className="mt-1 text-[11px] leading-relaxed text-[var(--text-tertiary)]">
            {t("userManagement.postgresOnlyReason")}
          </p>
        </div>
      ) : (
        <p className="py-1 text-xs text-[var(--text-tertiary)]">
          {t("userManagement.sidebarHint")}
        </p>
      )}
    </div>
  );
}
