import { useState } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/button";

import { useConnectionStore } from "@/commons/stores/connection.store";

import { CreateRoleDialog } from "../components/create-role-dialog";
import { UserDetailPanel } from "../components/user-detail-panel";
import { UserList } from "../components/user-list";
import {
  useCreateRole,
  useDropRole,
  useGrantPrivilege,
  useListPrivileges,
  useListUsers,
  useRevokePrivilege,
} from "../queries/user.queries";

export function UserManagementPage() {
  const { t } = useTranslation();
  const explorerConnectionId = useConnectionStore((s) => s.explorerConnectionId);
  const [selectedUser, setSelectedUser] = useState<string | null>(null);
  const [createOpen, setCreateOpen] = useState(false);

  const { data: users = [], isLoading } = useListUsers(explorerConnectionId);
  const { data: privileges = [] } = useListPrivileges(
    explorerConnectionId,
    selectedUser,
  );
  const createRole = useCreateRole(explorerConnectionId);
  const dropRole = useDropRole(explorerConnectionId);
  const grant = useGrantPrivilege(explorerConnectionId, selectedUser);
  const revoke = useRevokePrivilege(explorerConnectionId, selectedUser);

  if (!explorerConnectionId) {
    return (
      <div className="flex h-full items-center justify-center text-muted-foreground">
        {t("userManagement.connectFirst")}
      </div>
    );
  }

  const selectedUserData = users.find((u) => u.name === selectedUser) ?? null;

  return (
    <div className="flex h-full gap-4">
      <div className="flex w-64 shrink-0 flex-col gap-2 overflow-auto border-r border-border pr-4">
        <div className="flex items-center justify-between">
          <h2 className="text-sm font-semibold text-foreground">
            {t("userManagement.title")}
          </h2>
          <Button
            type="button"
            size="sm"
            onClick={() => setCreateOpen(true)}
          >
            + {t("userManagement.newRole")}
          </Button>
        </div>

        {isLoading ? (
          <p className="text-xs text-muted-foreground">
            {t("common.loading")}
          </p>
        ) : (
          <UserList
            users={users}
            selectedUser={selectedUser}
            onSelectUser={setSelectedUser}
            onDropRole={(name) => {
              if (confirm(t("userManagement.confirmDrop", { name }))) {
                dropRole.mutate(name);
                if (selectedUser === name) setSelectedUser(null);
              }
            }}
          />
        )}
      </div>

      <div className="flex-1 overflow-auto">
        {selectedUserData ? (
          <UserDetailPanel
            user={selectedUserData}
            privileges={privileges}
            onGrant={(schema, table, privilege) =>
              grant.mutate({ schema, table, privilege })
            }
            onRevoke={(schema, table, privilege) =>
              revoke.mutate({ schema, table, privilege })
            }
          />
        ) : (
          <div className="flex h-full items-center justify-center text-muted-foreground">
            {t("userManagement.selectRole")}
          </div>
        )}
      </div>

      <CreateRoleDialog
        open={createOpen}
        onClose={() => setCreateOpen(false)}
        onCreate={(name, login) => createRole.mutate({ name, login })}
      />
    </div>
  );
}
