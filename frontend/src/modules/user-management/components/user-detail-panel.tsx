import { useTranslation } from "react-i18next";

import type { DatabaseUser } from "../types/user.types";
import { PrivilegeManager } from "./privilege-manager";
import type { Privilege } from "../types/user.types";

interface UserDetailPanelProps {
  user: DatabaseUser;
  privileges: Privilege[];
  onGrant: (schema: string, table: string, privilege: string) => void;
  onRevoke: (schema: string, table: string, privilege: string) => void;
}

export function UserDetailPanel({
  user,
  privileges,
  onGrant,
  onRevoke,
}: UserDetailPanelProps) {
  const { t } = useTranslation();

  const attrs = [
    { label: t("userManagement.superuser"), value: user.isSuper },
    { label: t("userManagement.canLogin"), value: user.canLogin },
    { label: t("userManagement.canCreateDb"), value: user.canCreateDb },
    { label: t("userManagement.canCreateRole"), value: user.canCreateRole },
  ];

  return (
    <div className="flex flex-col gap-4">
      <h2
        className="text-lg font-semibold"
        style={{ color: "var(--color-text)" }}
      >
        {user.name}
      </h2>

      <div className="flex flex-wrap gap-2">
        {attrs.map((attr) => (
          <span
            key={attr.label}
            className="rounded-[var(--radius-sm)] px-2 py-1 text-xs"
            style={{
              backgroundColor: attr.value
                ? "var(--color-success-bg, var(--color-surface))"
                : "var(--color-surface)",
              color: attr.value
                ? "var(--color-success, var(--color-text))"
                : "var(--color-text-secondary)",
              border: "1px solid var(--color-border)",
            }}
          >
            {attr.label}: {attr.value ? t("common.yes") : t("common.no")}
          </span>
        ))}
      </div>

      <PrivilegeManager
        privileges={privileges}
        onGrant={onGrant}
        onRevoke={onRevoke}
      />
    </div>
  );
}
