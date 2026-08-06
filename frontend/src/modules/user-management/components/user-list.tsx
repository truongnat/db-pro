import { useTranslation } from "react-i18next";

import type { DatabaseUser } from "../types/user.types";

interface UserListProps {
  users: DatabaseUser[];
  selectedUser: string | null;
  onSelectUser: (name: string) => void;
  onDropRole: (name: string) => void;
}

export function UserList({
  users,
  selectedUser,
  onSelectUser,
  onDropRole,
}: UserListProps) {
  const { t } = useTranslation();

  return (
    <div className="flex flex-col gap-1">
      <h3
        className="px-2 py-1 text-xs font-semibold uppercase tracking-wide"
        style={{ color: "var(--color-text-secondary)" }}
      >
        {t("userManagement.roles")}
      </h3>
      {users.map((user) => (
        <div
          key={user.name}
          className="flex items-center justify-between rounded-[var(--radius-sm)] px-2 py-1.5 cursor-pointer transition-colors hover:bg-[var(--color-bg)]"
          style={{
            backgroundColor:
              selectedUser === user.name ? "var(--color-surface)" : undefined,
          }}
          onClick={() => onSelectUser(user.name)}
        >
          <div className="flex flex-col min-w-0">
            <span
              className="text-sm font-medium truncate"
              style={{ color: "var(--color-text)" }}
            >
              {user.name}
            </span>
            <span
              className="text-xs truncate"
              style={{ color: "var(--color-text-secondary)" }}
            >
              {user.isSuper
                ? t("userManagement.superuser")
                : user.canLogin
                  ? t("userManagement.login")
                  : t("userManagement.noLogin")}
            </span>
          </div>
          {!user.isSuper && (
            <button
              className="ml-2 shrink-0 rounded p-1 text-xs transition-colors hover:bg-[var(--color-border)]"
              style={{ color: "var(--color-error)" }}
              title={t("userManagement.dropRole")}
              onClick={(e) => {
                e.stopPropagation();
                onDropRole(user.name);
              }}
            >
              {t("common.actions.delete")}
            </button>
          )}
        </div>
      ))}
    </div>
  );
}
