import { useTranslation } from "react-i18next";

import { cn } from "@/lib/utils";

import { Button } from "@/components/ui/button";

import type { DatabaseUser } from "../types/user.types";

interface UserListProps {
  users: DatabaseUser[];
  selectedUser: string | null;
  onSelectUser: (name: string) => void;
  onDropRole: (name: string) => void;
}

export function UserList({ users, selectedUser, onSelectUser, onDropRole }: UserListProps) {
  const { t } = useTranslation();

  return (
    <div className="flex flex-col gap-1">
      <h3 className="px-2 py-1 text-xs font-semibold uppercase tracking-wide text-[var(--app-text-muted)]">
        {t("userManagement.roles")}
      </h3>
      {users.map((user) => (
        <div
          key={user.name}
          className={cn(
            "flex items-center justify-between rounded-sm px-2 py-1.5 cursor-pointer transition-colors hover:bg-background",
            selectedUser === user.name && "bg-background",
          )}
          onClick={() => onSelectUser(user.name)}
        >
          <div className="flex flex-col min-w-0">
            <span className="text-sm font-medium truncate text-foreground">{user.name}</span>
            <span className="text-xs truncate text-[var(--app-text-muted)]">
              {user.isSuper
                ? t("userManagement.superuser")
                : user.canLogin
                  ? t("userManagement.login")
                  : t("userManagement.noLogin")}
            </span>
          </div>
          {!user.isSuper && (
            <Button
              type="button"
              variant="ghost"
              size="xs"
              className="ml-2 shrink-0 text-destructive hover:bg-border"
              title={t("userManagement.dropRole")}
              onClick={(e) => {
                e.stopPropagation();
                onDropRole(user.name);
              }}
            >
              {t("common.actions.delete")}
            </Button>
          )}
        </div>
      ))}
    </div>
  );
}
