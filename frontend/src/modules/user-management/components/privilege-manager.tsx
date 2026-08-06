import { useState } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/button";

import type { Privilege } from "../types/user.types";

interface PrivilegeManagerProps {
  privileges: Privilege[];
  onGrant: (schema: string, table: string, privilege: string) => void;
  onRevoke: (schema: string, table: string, privilege: string) => void;
}

const PRIVILEGE_TYPES = ["SELECT", "INSERT", "UPDATE", "DELETE", "TRUNCATE", "REFERENCES", "TRIGGER"];

export function PrivilegeManager({
  privileges,
  onGrant,
  onRevoke,
}: PrivilegeManagerProps) {
  const { t } = useTranslation();
  const [schema, setSchema] = useState("public");
  const [table, setTable] = useState("");
  const [privType, setPrivType] = useState("SELECT");

  const handleGrant = () => {
    if (!table.trim()) return;
    onGrant(schema.trim(), table.trim(), privType);
    setTable("");
  };

  const grouped = new Map<string, Privilege[]>();
  for (const p of privileges) {
    const key = `${p.schema}.${p.table}`;
    const list = grouped.get(key) ?? [];
    list.push(p);
    grouped.set(key, list);
  }

  return (
    <div className="flex flex-col gap-4">
      <h3 className="text-sm font-semibold text-foreground">
        {t("userManagement.privileges")}
      </h3>

      <div className="flex flex-col gap-2">
        {Array.from(grouped.entries()).map(([key, privs]) => (
          <div
            key={key}
            className="rounded-sm border p-2 border-border"
          >
            <div className="mb-1 text-xs font-medium text-muted-foreground">
              {key}
            </div>
            <div className="flex flex-wrap gap-1">
              {privs.map((p) => (
                <span
                  key={p.privilegeType}
                  className="inline-flex items-center gap-1 rounded px-1.5 py-0.5 text-xs bg-card text-foreground"
                >
                  {p.privilegeType}
                  <Button
                    type="button"
                    variant="ghost"
                    size="xs"
                    className="ml-0.5 text-destructive"
                    title={t("userManagement.revoke")}
                    onClick={() => onRevoke(p.schema, p.table, p.privilegeType)}
                  >
                    x
                  </Button>
                </span>
              ))}
            </div>
          </div>
        ))}
        {privileges.length === 0 && (
          <p className="text-xs italic text-muted-foreground">
            {t("userManagement.noPrivileges")}
          </p>
        )}
      </div>

      <div className="flex flex-col gap-2 rounded-sm border border-border p-3">
        <span className="text-xs font-medium text-muted-foreground">
          {t("userManagement.grantPrivilege")}
        </span>
        <div className="flex gap-2">
          <input
            type="text"
            value={schema}
            onChange={(e) => setSchema(e.target.value)}
            placeholder="schema"
            className="w-24 rounded-sm border border-border bg-card px-2 py-1 text-xs text-foreground"
          />
          <input
            type="text"
            value={table}
            onChange={(e) => setTable(e.target.value)}
            placeholder="table"
            className="w-28 rounded-sm border border-border bg-card px-2 py-1 text-xs text-foreground"
          />
          <select
            value={privType}
            onChange={(e) => setPrivType(e.target.value)}
            className="rounded-sm border border-border bg-card px-2 py-1 text-xs text-foreground"
          >
            {PRIVILEGE_TYPES.map((pt) => (
              <option key={pt} value={pt}>
                {pt}
              </option>
            ))}
          </select>
          <Button
            type="button"
            size="sm"
            onClick={handleGrant}
            disabled={!table.trim()}
          >
            {t("userManagement.grant")}
          </Button>
        </div>
      </div>
    </div>
  );
}
