import { useEffect, useState } from "react";

import { useTranslation } from "@/commons/locales/useTranslation";
import { useConnectionStore } from "@/commons/stores/connection.store";
import { useConnectionList } from "@/modules/connection/queries/connection.queries";
import {
  useDropRole,
  useListPrivileges,
  useListUsers,
} from "@/modules/user-management/queries/user.queries";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";

export function UsersView() {
  const { t } = useTranslation();
  const connections = useConnectionList();
  const explorerConnectionId = useConnectionStore((s) => s.explorerConnectionId);
  const selectedConnection = connections.data?.find((c) => c.id === explorerConnectionId);
  const postgresConnectionId =
    selectedConnection?.driver === "postgres" ? selectedConnection.id : null;
  const users = useListUsers(postgresConnectionId);
  const [selectedRole, setSelectedRole] = useState<string | null>(null);
  const [pendingDrop, setPendingDrop] = useState<string | null>(null);
  const privileges = useListPrivileges(postgresConnectionId, selectedRole);
  const dropRole = useDropRole(postgresConnectionId);

  useEffect(() => {
    setSelectedRole((current) => {
      if (current && users.data?.some((user) => user.name === current)) return current;
      return users.data?.[0]?.name ?? null;
    });
  }, [users.data]);

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
        <>
          {users.isPending && (
            <p className="py-1 text-xs text-[var(--text-tertiary)]">{t("common.states.loading")}</p>
          )}
          {users.isError && (
            <p className="py-1 text-xs text-destructive">{t("common.states.error")}</p>
          )}
          {!users.isPending && !users.isError && users.data?.length === 0 && (
            <p className="py-1 text-xs text-[var(--text-tertiary)]">{t("common.states.empty")}</p>
          )}
          {users.data && users.data.length > 0 && (
            <div className="flex min-h-0 flex-col gap-1">
              <span className="px-1 text-[11px] font-medium text-[var(--text-secondary)]">
                {t("userManagement.roles")}
              </span>
              <div className="flex max-h-56 flex-col gap-0.5 overflow-y-auto">
                {users.data.map((user) => (
                  <div
                    key={user.name}
                    className={`flex items-center gap-1 rounded-md px-1.5 py-1 ${
                      selectedRole === user.name ? "bg-[var(--surface-active)]" : ""
                    }`}
                  >
                    <button
                      type="button"
                      className="min-w-0 flex-1 truncate text-left text-xs text-foreground hover:text-primary"
                      onClick={() => setSelectedRole(user.name)}
                    >
                      {user.name}
                    </button>
                    <span className="text-[10px] text-[var(--text-tertiary)]">
                      {user.canLogin ? t("userManagement.login") : t("userManagement.noLogin")}
                    </span>
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      className="h-6 px-1.5 text-[10px] text-destructive hover:text-destructive"
                      onClick={() => setPendingDrop(user.name)}
                    >
                      {t("userManagement.dropRole")}
                    </Button>
                  </div>
                ))}
              </div>
              {selectedRole && (
                <p className="px-1 text-[11px] text-[var(--text-tertiary)]">
                  {t("userManagement.privilegeCount", { count: privileges.data?.length ?? 0 })}
                </p>
              )}
            </div>
          )}
        </>
      )}

      <AlertDialog
        open={pendingDrop !== null}
        onOpenChange={(open) => !open && setPendingDrop(null)}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t("userManagement.dropRole")}</AlertDialogTitle>
            <AlertDialogDescription>
              {pendingDrop &&
                t("userManagement.confirmDropImpact", {
                  name: pendingDrop,
                  count: privileges.data?.length ?? 0,
                })}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t("common.actions.cancel")}</AlertDialogCancel>
            <AlertDialogAction
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
              disabled={dropRole.isPending}
              onClick={() => {
                if (pendingDrop) dropRole.mutate(pendingDrop);
                setPendingDrop(null);
              }}
            >
              {t("userManagement.dropRole")}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}
